import path from "path";
import fs from "fs";
import { spawnSync } from "child_process";
import { fileURLToPath } from "url";

const pluginDir = path.dirname(fileURLToPath(import.meta.url));
const skillsDir = path.resolve(pluginDir, "../../skills");

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

function loadSkills() {
  const commands = {};
  let entries;
  try {
    entries = fs.readdirSync(skillsDir);
  } catch {
    return commands;
  }
  for (const entry of entries) {
    let content;
    try {
      content = fs.readFileSync(path.join(skillsDir, entry, "SKILL.md"), "utf8");
    } catch {
      continue;
    }
    const fields = parseFrontmatter(content);
    if (!fields || !fields.name) continue;
    if (fields["user-invocable"] === "false") continue;
    const command = {
      template: `Load and execute the \`${fields.name}\` skill.\n\n$ARGUMENTS`,
    };
    if (fields.description) command.description = fields.description;
    commands[fields.name] = command;
  }
  return commands;
}

const skillCommands = loadSkills();

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

export const CompoundEngineeringPlugin = async ({ project, client, $, directory, worktree }) => {
  const cwd = directory || worktree || process.cwd();

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

export default CompoundEngineeringPlugin;
