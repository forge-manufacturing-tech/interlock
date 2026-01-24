# AGENTS Instructions

If you are a new agent instance working on this repository, you **MUST** follow these rules:

1.  **Read the READMEs**: Always start by reading:
    -   `./README.md`
    -   `backend/README.md`
    -   `frontend/README.md`
    These files contain the source of truth for workflows, architecture, and commands.
2. **You are in a dev container**: Also read through `.devcontainer/devcontainer.json` to see what is installed in the environment.
3. **Database Setup**: In the dev container, see `.devcontainer/setup.sh` to see how the database is setup intially
4. **Deployment on Github actions**: Actual deployment logic and environment variables are stored in Github actions. Do not modify them. You can see the workflow in `.github/workflows/deploy-backend.yml`