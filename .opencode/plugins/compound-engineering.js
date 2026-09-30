import path from "path";
import fs from "fs";
import { spawnSync } from "child_process";
import { fileURLToPath } from "url";

const pluginDir = path.dirname(fileURLToPath(import.meta.url));
const candidateDirs = [
  path.resolve(pluginDir, "../../skills"),
  path.resolve(pluginDir, "../compound-engineering/skills"),
  path.resolve(pluginDir, "../skills"),
];
const skillsDir = candidateDirs.find((d) => fs.existsSync(d)) || candidateDirs[0];

function unquote(value) {
  if (value.length < 2) return value;
  const quote = value[0];
  if ((quote !== '"' && quote !== "'") || value[value.length - 1] !== quote) return value;
  const inner = value.slice(1, -1);
  return quote === '"' ? inner.replace(/\\(["\\])/g, "$1") : inner.replace(/''/g, "'");
}

// Scoped to the leading `---` block so a `name:`/`description:` line inside a
// fenced YAML example in the skill body cannot register a bogus command.
function parseFrontmatter(content) {
  const block = content.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!block) return null;
  const fields = {};
  for (const line of block[1].split(/\r?\n/)) {
    const pair = line.match(/^([A-Za-z][\w-]*):\s*(.*)$/);
    if (pair) fields[pair[1]] = unquote(pair[2].trim());
  }
  return fields;
}

/**
 * Discovers CE skills in `skillsDir`. Every directory holding a SKILL.md
 * becomes a registered OpenCode skill; `user-invocable: false` entries are
 * still registered as skills but receive no slash command (V1 loader parity).
 */
function loadSkills() {
  const skills = [];
  let entries;
  try {
    entries = fs.readdirSync(skillsDir);
  } catch {
    return skills;
  }
  for (const entry of entries) {
    const file = path.join(skillsDir, entry, "SKILL.md");
    let content;
    try {
      content = fs.readFileSync(file, "utf8");
    } catch {
      continue;
    }
    const fields = parseFrontmatter(content);
    if (!fields || !fields.name) continue;
    skills.push({
      id: fields.name,
      name: fields.name,
      description: fields.description || "",
      userInvocable: fields["user-invocable"] !== "false",
      file,
      content,
    });
  }
  return skills;
}

/**
 * Executes `ce-ai workflow resume` in the session's workspace directory.
 * Returns the live environment state text, or null if execution failed.
 */
function getRepoState(cwd) {
  try {
    const res = spawnSync("ce-ai", ["workflow", "resume"], {
      cwd: cwd || process.cwd(),
      encoding: "utf8",
      timeout: 5000,
      env: process.env,
    });
    if (res && res.status === 0 && res.stdout) {
      return res.stdout.trim();
    }
  } catch {
    // Fail gracefully if ce-ai is not on PATH or execution fails
  }
  return null;
}

function skillDefinition(skill) {
  const definition = {
    id: skill.id,
    name: skill.name,
    path: skill.file,
    content: skill.content,
  };
  if (skill.description) definition.description = skill.description;
  return definition;
}

/**
 * OpenCode V1 plugin implementation.
 * V1 calls this function (either as named export or as default.server).
 */
export const CompoundEngineeringPlugin = async ({ project, client, $, directory, worktree }) => {
  const cwd = directory || worktree || process.cwd();
  const skills = loadSkills();
  const skillCommands = {};
  for (const skill of skills) {
    if (!skill.userInvocable) continue;
    skillCommands[skill.id] = {
      template: `Load and execute the \`${skill.id}\` skill.\n\n$ARGUMENTS`,
      ...(skill.description ? { description: skill.description } : {}),
    };
  }

  return {
    config: async (config) => {
      config.skills = config.skills || {};
      config.skills.paths = config.skills.paths || [];
      if (!config.skills.paths.includes(skillsDir)) {
        config.skills.paths.push(skillsDir);
      }
      config.command = config.command || {};
      for (const [name, cmd] of Object.entries(skillCommands)) {
        if (!(name in config.command)) {
          config.command[name] = cmd;
        }
      }
    },

    event: async ({ event }) => {
      if (event && event.type === "session.created") {
        const sessionId =
          event.properties?.info?.id ||
          event.properties?.sessionID ||
          event.sessionID;

        const stateOutput = getRepoState(cwd);
        if (sessionId && stateOutput && client && client.session && typeof client.session.prompt === "function") {
          try {
            await client.session.prompt({
              path: { id: sessionId },
              body: {
                noReply: true,
                parts: [{ type: "text", text: stateOutput }],
              },
            });
          } catch {
            // Non-blocking: continue normal session execution if prompt injection fails
          }
        }
      } else if (event && event.type === "session.idle") {
        // Turn-end auto-checkpoint: invoke ce-ai workflow resume to evaluate stage progression
        getRepoState(cwd);
      }
    },

    "experimental.session.compacting": async (input, output) => {
      const stateOutput = getRepoState(cwd);
      if (stateOutput && output && Array.isArray(output.context)) {
        output.context.push(stateOutput);
      }
    },

    "experimental.chat.system.transform": async (input, output) => {
      const stateOutput = getRepoState(cwd);
      if (stateOutput && output && Array.isArray(output.system)) {
        output.system.push(stateOutput);
      }
    },
  };
};

/**
 * OpenCode V1/V2 Dual Plugin Definition.
 * - OpenCode V1 invokes `server()` (and has named export `CompoundEngineeringPlugin`).
 * - OpenCode V2 invokes `setup(ctx)` with `id: "compound-engineering"`.
 */
export default {
  id: "compound-engineering",
  server: CompoundEngineeringPlugin,

  async setup(ctx) {
    const cwd = (ctx.location && ctx.location.directory) || process.cwd();
    const skills = loadSkills();

    // Skills: register every discovered SKILL.md. Idempotent when the same
    // id was already provided (e.g. via the managed `skills.paths` config).
    await ctx.skill.transform((editor) => {
      for (const skill of skills) {
        if (editor.get(skill.id)) {
          editor.update(skill.id, (existing) => {
            existing.name = skill.name;
            existing.path = skill.file;
            existing.content = skill.content;
            if (skill.description) existing.description = skill.description;
          });
        } else {
          editor.add(skillDefinition(skill));
        }
      }
    });

    // Commands: one invocable slash command per user-invocable skill,
    // preserving the V1 template semantics (`$ARGUMENTS` := prompt text).
    // Names already taken (e.g. user-configured commands) keep precedence.
    const existingCommands = new Set();
    try {
      const available = await ctx.command.list();
      for (const command of Array.isArray(available) ? available : []) {
        if (command && command.name) existingCommands.add(command.name);
      }
    } catch {
      // Registry read is best-effort; registration below still applies.
    }
    await ctx.command.transform((editor) => {
      for (const skill of skills) {
        if (!skill.userInvocable || existingCommands.has(skill.id)) continue;
        editor.add({
          name: skill.id,
          ...(skill.description ? { description: skill.description } : {}),
          execute: async ({ sessionID, prompt, delivery }) => {
            const args = (prompt && typeof prompt.text === "string" && prompt.text) || "";
            const text = `Load and execute the \`${skill.id}\` skill.\n\n${args}`.trimEnd();
            await ctx.session.prompt({ sessionID, text, delivery });
          },
        });
      }
    });

    // State delivery into assembled model requests: the agent loop
    // ("context") and checkpoint summaries ("compaction"). Fresh state is
    // therefore present on the first post-compaction request as well.
    const injectSystemState = (event) => {
      const stateOutput = getRepoState(cwd);
      if (stateOutput && event && Array.isArray(event.system)) {
        event.system.push({ type: "text", text: stateOutput });
      }
    };
    await ctx.session.hook("context", injectSystemState);
    await ctx.session.hook("compaction", injectSystemState);

    // Session lifecycle: Turn-0 state injection (synthetic context message,
    // the V2 replacement for noReply prompts) and turn-end FSM checkpoints.
    const controller = new AbortController();
    void (async () => {
      try {
        for await (const event of ctx.event.subscribe({ signal: controller.signal })) {
          if (event.type === "session.created") {
            const sessionId =
              event.sessionID ||
              (event.properties &&
                (event.properties.sessionID ||
                  (event.properties.info && event.properties.info.id)));
            const stateOutput = getRepoState(cwd);
            if (sessionId && stateOutput) {
              try {
                await ctx.session.synthetic({ sessionID: sessionId, text: stateOutput });
              } catch {
                // Non-blocking: continue normal session execution if injection fails
              }
            }
          } else if (event.type === "session.idle") {
            // Turn-end auto-checkpoint: invoke ce-ai workflow resume to evaluate stage progression
            getRepoState(cwd);
          }
        }
      } catch {
        // Subscription aborted during plugin unload.
      }
    })();

    return () => controller.abort();
  },
};
