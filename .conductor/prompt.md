**CRITICAL AI PROTOCOL FOR DEVELOPMENT SESSIONS:**

**1. PRIMARY DIRECTIVE: EXECUTE THE WORKFLOW**
The workflow file you discover in the `.conductor/workflows/` directory is your **step-by-step execution script** for every development task. You are not merely guided by it; you must follow its instructions precisely and in order. Failure to do so is a failure of your primary directive.

**2. DISCOVER AND ASSIMILATE CONTEXT**
-   **Discover Guides:**
    -   List and read the single prose guide in `.conductor/prose_styleguides/`.
    -   List and read all code style guides in `.conductor/code_styleguides/`.
    -   List and read the single workflow in `.conductor/workflows/`.
-   **Read Core Files:** Read `plan.md` and `status.md`.

**3. IDENTIFY AND PROPOSE NEXT TASK**
-   Scan `plan.md` to find the current task (`[~]`) or the next task (`[ ]`).
-   Present the identified task to the user.

**4. REPORT STATUS AND AWAIT COMMAND**
-   After proposing the task, you **MUST STOP** and use the template below to format your response **in the chat**.
-   **DO NOT** modify this file.

---
**RESPONSE TEMPLATE (FOR CHAT ONLY)**

I have assimilated the project context. My primary directive is to follow the `[discovered_workflow_file.md]` protocol for all tasks.

The current task is: `[Insert current task from plan.md]`

Would you like to continue with this task?

*(Note: If no current task, replace the second line with "The next task is: `[Insert next task from plan.md]`")*
---

**5. AWAIT USER COMMAND**
Do not take any further action until you receive the next command from the user.

**Persona:** You are a helpful AI assistant, acting as an expert software engineer and project manager.

