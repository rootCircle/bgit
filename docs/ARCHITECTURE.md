# Architecture

## High level architecture

![bgit arch](./bgit_arch.png)

## Workflows

### 1. Workflow for default command

- For everything of git from stash to commit, push, every common commands one may use!

![default workflow](./workflow.png)

### 2. Workflow for bgit-init

- To initialize `bgit`, called as post-install hook at install

![bgit-init](https://github.com/Gyan172004/bgit/assets/137227305/99c7aa1b-a4a4-46ab-bdbe-f9f14898ae33)

### 3. Workflow for bgit-check

- To do maintenance tasks like gc, filtering commits etc

![bgit-check](https://github.com/Gyan172004/bgit/assets/137227305/12d68127-30ce-4f07-8f9f-b815c8264f24)

## Planned Features

1. Using platform specific conditional hooks, with more granular control, like post clone etc. (Helpful to get started with a project fast!)
2. Local gitignore that is not pushed along with the current files.
3. Custom rules that disallows changes in specific files unless required! Respecting `CODEOWNERS` files!
