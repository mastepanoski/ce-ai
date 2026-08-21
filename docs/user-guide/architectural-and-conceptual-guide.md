# 🏛️ Guía Arquitectónica y Conceptual de `ce-ai`

Esta guía aborda en profundidad la **arquitectura de software, patrones de diseño y conceptos de ingeniería** sobre los cuales está construido `ce-ai`. Su objetivo es explicar el *cómo* y el *porqué* arquitectónico detrás de cada decisión del sistema.

---

## 1. Capa de Orquestación Multi-Arnés (Multi-Harness Orchestration Architecture)

### 📐 El Concepto Arquitectónico
`ce-ai` está diseñado bajo un patrón de **desacoplamiento total de adaptadores** (`HarnessAdapter` trait en Rust).

```
                      ┌─────────────────────────┐
                      │    ce-ai CLI Engine     │
                      └────────────┬────────────┘
                                   │
                      ┌────────────┴────────────┐
                      │ HarnessAdapter (Trait)  │
                      └────┬───────────────┬────┘
                           │               │
        ┌──────────────────┴──┐         ┌──┴──────────────────┐
        │ ClaudeCodeAdapter   │         │  OpenCodeAdapter    │
        │ (.claude.json)      │         │  (opencode.json)    │
        └─────────────────────┘         └─────────────────────┘
        ┌─────────────────────┐         ┌─────────────────────┐
        │ CursorAdapter       │         │ GenericJsonAdapter  │
        │ (.cursorrules)      │         │ (Pi, Kimi, AGY...)  │
        └─────────────────────┘         └─────────────────────┘
```

### 💡 Justificación Arquitectónica
- **Abstracción de Interfaces**: Cada herramienta de IA (Claude Code, OpenCode, Cursor, Copilot, Kimi, Antigravity) maneja formatos de archivo y esquemas de configuración totalmente heterogéneos (JSON, Markdown con delimitadores, estructuras de arreglos de plugins).
- **Principio de Responsabilidad Única (SRP)**: El motor central de `ce-ai` no conoce los detalles internos de sintaxis de cada editor. Simplemente delega en el `HarnessAdapter` correspondiente la tarea de fusionar la configuración de forma no destructiva.
- **Extensibilidad Segura**: Agregar soporte para un nuevo editor o arnés de IA requiere únicamente implementar un nuevo adaptador que satisfaga el trait `HarnessAdapter`, sin arriesgar regresiones en el motor principal.

---

## 2. Aislamiento de Capas y Árbol de Ámbito (Scope Isolation: Global vs Workspace)

### 📐 El Concepto Arquitectónico
El sistema aplica un patrón de **Jerarquía de Configuración y Aislamiento de Ámbito** (*Scope-Aware Hierarchy*).

- **Global Scope (`~/.config/` / `~/.claude.json`)**: Capa de usuario donde residen las preferencias personales y herramientas transversales a cualquier máquina o repositorio.
- **Workspace Scope (`./.opencode/` / `./.cursorrules`)**: Capa de repositorio acotada al árbol de trabajo de Git, resuelta determinísticamente mediante `git rev-parse --show-toplevel`.

### 💡 Justificación Arquitectónica
- **Prevención de Contaminación Cruzada (Rule Cross-Contamination)**: Proyectos distintos poseen diferentes arquitecturas, restricciones de seguridad y convenciones de código. Si las reglas de IA fuesen únicamente globales, las directivas de un proyecto backend en Rust contaminarían la sesión de un proyecto frontend en React.
- **Reproducibilidad en Equipo**: El ámbito *workspace* permite versionar dentro del repositorio Git las habilidades y reglas del proyecto. Cualquier desarrollador o agente que clone el repositorio adquiere automáticamente el mismo contexto operativo exacto.

---

## 3. Arquitectura de Sidecars y Servidores de Conocimiento (Sidecars & MCP Protocol)

### 📐 El Concepto Arquitectónico
`ce-ai` utiliza el patrón de diseño **Sidecar (Procesos Acompañantes)** junto con la infraestructura de **Model Context Protocol (MCP)**.

```
┌─────────────────────────────────────────────────────────────────┐
│                    Agente Ejecutor de IA                        │
└──────┬──────────────────────┬──────────────────────┬────────────┘
       │ (MCP)                │ (MCP)                │ (MCP)
┌──────┴──────────────┐ ┌─────┴──────────────┐ ┌─────┴──────────────┐
│  Engram Memory      │ │ CodeGraph Engine   │ │ Context7 / RTK     │
│  (Persistencia)     │ │ (Grafo de Código)  │ │ (Docs en Vivo)     │
└─────────────────────┘ └────────────────────┘ └────────────────────┘
```

- **Engram**: Microservicio de almacenamiento duradero que persiste descubrimientos, decisiones y soluciones entre sesiones.
- **CodeGraph**: Motor de indexación estática y análisis de impacto (*blast-radius*) que expone la topología del código.
- **Context7 & RTK**: Proveedores de contexto dinámico y documentación en tiempo real.

### 💡 Justificación Arquitectónica
- **Desacoplamiento entre Razonamiento y Almacenamiento**: Los Modelos de Lenguaje (LLMs) son excelentes procesando razonamiento en tiempo presente, pero deficientes almacenando estructuras duraderas a largo plazo.
- **Eficiencia de la Ventana de Contexto**: En lugar de cargar todo el código fuente o la historia pasada dentro del prompt, el proceso Sidecar responde a consultas puntuales bajo demanda via MCP, reduciendo el consumo de tokens y evitando saturar la atención del modelo.

---

## 4. Máquinas de Estados Finitos y Resiliencia de Flujo (Workflow FSM & Checkpointing)

### 📐 El Concepto Arquitectónico
El desarrollo asistido por IA en `ce-ai` está gobernado por una **Máquina de Estados Finitos (FSM)** que estructura el ciclo de vida en 7 fases deterministas:

$$\text{Ideación} \xrightarrow{1} \text{OpenSpec} \xrightarrow{2} \text{Planificación} \xrightarrow{3} \text{Trabajo/TDD} \xrightarrow{4} \text{Verificación} \xrightarrow{5} \text{Documentación} \xrightarrow{6} \text{Publicación}$$

### 💡 Justificación Arquitectónica
- **Determinismo vs. Probabilidad**: La generación de código con LLMs es probabilística. Sin un motor de estados que exija pasar por especificación (`OpenSpec`), plan (`Plan`) y pruebas (`TDD`), el proceso degenera en parches superficiales o código inconsistente.
- **Checkpointing y Re-hidratación de Contexto**:
  - *Problema*: Durante tareas largas, la ventana de contexto del LLM sufre compactación (pérdida de memoria).
  - *Solución Arquitectónica*: `ce-ai workflow checkpoint` serializa en disco el estado exacto de la FSM y la tarea activa. Al reiniciar o cambiar de agente, `ce-ai workflow resume` lee el disco y re-hidrata la memoria sin duplicar trabajo ni perder la traza.

---

## 5. Resiliencia de E/S y Tolerancia a Fallos (System Integrity & Fault Tolerance)

### 📐 El Concepto Arquitectónico

1. **Escrituras Atómicas (`write_atomic`)**:
   - Para evitar corrupción de configuración (`state.json`, `opencode.json`), las escrituras nunca modifican el archivo destino de forma directa. Se escribe un archivo borrador temporal (`.tmp`) en el mismo sistema de archivos y se ejecuta una operación de renombrado atómico del kernel POSIX/OS.

2. **Detección de Desviación criptográfica (SHA256 Drift Detection)**:
   - `install-manifest.json` mantiene un índice hash SHA256 de cada habilidad. El motor de reconciliación (`sync`) calcula el diff criptográfico de tres vías:
     - **Copy**: Archivos faltantes en disco.
     - **Restore**: Archivos locales alterados respecto al manifest de referencia.
     - **Remove**: Archivos obsoletos o extirpados de la versión de origen.

3. **Invariante de Preservación del Usuario (Non-Destructive Merger)**:
   - El sistema garantiza que ninguna mutación de `ce-ai` elimine claves, servidores MCP o configuraciones personalizadas del usuario.

---

## 📊 Matriz de Conceptos y Pilares Arquitectónicos

| Pilar Arquitectónico | Patrón / Mecanismo | Problema del Sistema que Resuelve |
| :--- | :--- | :--- |
| **Adaptabilidad** | `HarnessAdapter` (Traits) | Heterogeneidad de editores de IA y esquemas de configuración. |
| **Aislamiento** | Hierarchical Scope (Global vs Workspace) | Contaminación cruzada de directivas entre distintos proyectos. |
| **Persistencia Externa** | Sidecars & MCP (Engram / CodeGraph) | Saturación de la ventana de contexto y pérdida de memoria entre sesiones. |
| **Determinismo de Flujo** | Workflow FSM & Checkpointing | Inestabilidad probabilística y degradación por compactación de contexto. |
| **Tolerancia a Fallos** | Atomic Writes & SHA256 Manifest Index | Corrupción de archivos por interrupciones o ediciones accidentales en disco. |
