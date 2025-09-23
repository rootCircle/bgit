# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.4.1 (2025-09-24)

### Other

 - <csr-id-51e13e5777f2dbe7ba247a626c31bc537c3fb676/> upgrade deps

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 8 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Upgrade deps ([`51e13e5`](https://github.com/rootCircle/bgit/commit/51e13e5777f2dbe7ba247a626c31bc537c3fb676))
</details>

## v0.4.0 (2025-09-15)

<csr-id-21e50692156c95ec1b872a561715c5353ee087bc/>
<csr-id-913aab2b8a934be1a7a717b2d9107fe2fc2e3b63/>
<csr-id-6e0b8274026a8a4ee92c2189d2be55bb0e631110/>

### Chore

 - <csr-id-21e50692156c95ec1b872a561715c5353ee087bc/> update bgit to v0.4
 - <csr-id-913aab2b8a934be1a7a717b2d9107fe2fc2e3b63/> update ssh agent socket basename
   - Updated comments to reflect the change and clarify platform-specific behavior.
 - <csr-id-6e0b8274026a8a4ee92c2189d2be55bb0e631110/> add AGENTS.md

### Documentation

 - <csr-id-ea529bbc8b3720c0317554a7d078b5563867273b/> Update AGENTS.md with logging conventions

### New Features

<csr-id-803f71f3e7562857f18d9644106eba3aec7bf463/>

 - <csr-id-23ea72525e03bd8d74354b686e7b540ba560ba4d/> Improve agent authentication and key management
   - Enhance SSH agent authentication to explicitly use configured agent socket.
- Allow interactive addition of SSH keys when agent authentication fails.
- Refactor SSH utility functions to accept explicit agent configuration.
- Improve handling of stale SSH agent sockets on Unix-like systems.
- Introduce functions for managing bgit-specific SSH agent state.
- Set global SSH environment variables for libgit2 compatibility.
- Define a `SshAgentManager` trait for platform-specific agent behaviors.
- Implement `UnixSshAgentManager` and `WindowsSshAgentManager` with platform-specific logic.
- Consolidate common SSH utilities (key adding, agent checks) into `ssh_utils.rs`.
- Update `src/auth/ssh/mod.rs` to re-export functions based on the target platform.
- Ensure `ensure_agent_ready` and `start_agent_detached` are available through the platform-agnostic interface.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release.
 - 27 days passed between releases.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release bgit v0.4.0 ([`b5ddfe4`](https://github.com/rootCircle/bgit/commit/b5ddfe4a2cc4a0cd9923c48075ff5173e11b1dee))
    - Update bgit to v0.4 ([`21e5069`](https://github.com/rootCircle/bgit/commit/21e50692156c95ec1b872a561715c5353ee087bc))
    - Update ssh agent socket basename ([`913aab2`](https://github.com/rootCircle/bgit/commit/913aab2b8a934be1a7a717b2d9107fe2fc2e3b63))
    - Update AGENTS.md with logging conventions ([`ea529bb`](https://github.com/rootCircle/bgit/commit/ea529bbc8b3720c0317554a7d078b5563867273b))
    - Fix(ssh): Validate environment SSH_AUTH_SOCK and PID Add validation for the SSH_AUTH_SOCK environment variable on Unix. The agent socket must exist, be a valid socket file, and respond to authentication requests. If the environment variables are invalid or the agent is unresponsive, they will be ignored. ([`7c4013b`](https://github.com/rootCircle/bgit/commit/7c4013b59d98f54b8523bdbd71e3de05cd71442f))
    - Improve agent authentication and key management ([`23ea725`](https://github.com/rootCircle/bgit/commit/23ea72525e03bd8d74354b686e7b540ba560ba4d))
    - Add AGENTS.md ([`6e0b827`](https://github.com/rootCircle/bgit/commit/6e0b8274026a8a4ee92c2189d2be55bb0e631110))
    - Introduce platform-agnostic SSH agent management ([`803f71f`](https://github.com/rootCircle/bgit/commit/803f71f3e7562857f18d9644106eba3aec7bf463))
    - Refactor SSH authentication handling ([`65c7d20`](https://github.com/rootCircle/bgit/commit/65c7d204a8285687c3c1a9f1a65aad28f03542c3))
</details>

<csr-unknown>
 Introduce platform-agnostic SSH agent managementRefactor SSH authentication logic to support different platforms by introducinga trait-based approach for SSH agent management.<csr-unknown/>

## v0.3.7 (2025-08-18)

### Bug Fixes

 - <csr-id-d1f54eb3cb9313fed8a5b479459e260dd2be0119/> remove stale ssh-agent socket if agent is dead
 - <csr-id-a332e8a0df8c619b828fec950602f1f38abf6bda/> broken links

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release bgit v0.3.7 ([`9991c47`](https://github.com/rootCircle/bgit/commit/9991c47e94287e7d5ebcf44c89483b0e30ea9654))
    - Remove stale ssh-agent socket if agent is dead ([`d1f54eb`](https://github.com/rootCircle/bgit/commit/d1f54eb3cb9313fed8a5b479459e260dd2be0119))
    - Broken links ([`a332e8a`](https://github.com/rootCircle/bgit/commit/a332e8a0df8c619b828fec950602f1f38abf6bda))
</details>

## v0.3.6 (2025-08-17)

### Bug Fixes

 - <csr-id-b2678e0c4a3c315fcc755ff5df4fd6441f4382de/> duplicate key files from global_config

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 day passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release bgit v0.3.6 ([`296e743`](https://github.com/rootCircle/bgit/commit/296e7438203877a862e5fab4e2fe9d79d2704d0d))
    - Duplicate key files from global_config ([`b2678e0`](https://github.com/rootCircle/bgit/commit/b2678e0c4a3c315fcc755ff5df4fd6441f4382de))
</details>

## v0.3.5 (2025-08-16)

### Bug Fixes

 - <csr-id-fdb1ee6b22dda536ba08580f05cbb9425152c24a/> Suspend progress bar for clean stderr output
   - Wrapped prompt_step_task.execute in self.pb.suspend to prevent
   dialoguer prompts from interfering with the progress bar.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release bgit v0.3.5 ([`29504f3`](https://github.com/rootCircle/bgit/commit/29504f3057840c3cc8fbcc5d67dd40ecf1c83f59))
    - Suspend progress bar for clean stderr output ([`fdb1ee6`](https://github.com/rootCircle/bgit/commit/fdb1ee6b22dda536ba08580f05cbb9425152c24a))
</details>

## v0.3.4 (2025-08-16)

<csr-id-125c1b6c6e20d242e71bea99f5f022395fda16e6/>
<csr-id-1af265e0a24b3242fa95a086599f887ce0ccfda9/>

### Chore

 - <csr-id-125c1b6c6e20d242e71bea99f5f022395fda16e6/> fix install script for mac

### New Features

<csr-id-4c315bbc5935957675a49c9d8ed34d6042c1263a/>

 - <csr-id-23853d66277d43dd6a7037a16e8bfaf76bfded6c/> auth.preffered integration and auth.ssh save
 - <csr-id-5e3f9716bb5948243b70a05f50ccd7784a2130a4/> prompt to persist SSH key and update config
   - Add interactive prompt to persist SSH key to global config.

### Bug Fixes

 - <csr-id-21aedabe72fb24cf16b4bfa498cc646de73f7a5a/> ensure formatting after clippy fix

### Refactor

 - <csr-id-1af265e0a24b3242fa95a086599f887ce0ccfda9/> introduce global configs

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release bgit v0.3.4 ([`563fe6d`](https://github.com/rootCircle/bgit/commit/563fe6ddcbfd4542fc30f6bf3afa23369313cd2f))
    - Ensure formatting after clippy fix ([`21aedab`](https://github.com/rootCircle/bgit/commit/21aedabe72fb24cf16b4bfa498cc646de73f7a5a))
    - Auth.preffered integration and auth.ssh save ([`23853d6`](https://github.com/rootCircle/bgit/commit/23853d66277d43dd6a7037a16e8bfaf76bfded6c))
    - Prompt to persist SSH key and update config ([`5e3f971`](https://github.com/rootCircle/bgit/commit/5e3f9716bb5948243b70a05f50ccd7784a2130a4))
    - Auth.https integration ([`4c315bb`](https://github.com/rootCircle/bgit/commit/4c315bbc5935957675a49c9d8ed34d6042c1263a))
    - Introduce global configs ([`1af265e`](https://github.com/rootCircle/bgit/commit/1af265e0a24b3242fa95a086599f887ce0ccfda9))
    - Fix install script for mac ([`125c1b6`](https://github.com/rootCircle/bgit/commit/125c1b6c6e20d242e71bea99f5f022395fda16e6))
</details>

## v0.3.3 (2025-08-16)

<csr-id-86564edfc5eb4d52b9ebbee5e7f41fdb8d288cfd/>
<csr-id-9234c37f487cd47bf93e0d04d3535ff0a2ccf355/>

### Documentation

<csr-id-178e216e48ecab71464b2750863a90a0ea1b6f4c/>

 - <csr-id-377aeb54882c7690badd28a42f3493b54e9c9eda/> Add HOOKS.md and improve hook execution
   - Document bgit's hook system, including portable (`.bgit/hooks`) and native Git hooks.

### New Features

 - <csr-id-e39ff22ae46c44be621b1df0a594539c0dcd9f35/> Add cross compilation and artifact packaging for more targets
   - Install `cross` for Linux builds.

### Refactor

 - <csr-id-86564edfc5eb4d52b9ebbee5e7f41fdb8d288cfd/> Improve error messages and tips for adding remotes

### Bug Fixes

 - <csr-id-74601c7cd5f56dd9c48d2415ad490349c157986f/> release builds
 - <csr-id-28b5d0d9a3fd91f74146e8c8883c71bb75c3040f/> musl targets builds

### Chore

 - <csr-id-9234c37f487cd47bf93e0d04d3535ff0a2ccf355/> cleanup hooks

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release over the course of 3 calendar days.
 - 3 days passed between releases.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release builds ([`74601c7`](https://github.com/rootCircle/bgit/commit/74601c7cd5f56dd9c48d2415ad490349c157986f))
    - Musl targets builds ([`28b5d0d`](https://github.com/rootCircle/bgit/commit/28b5d0d9a3fd91f74146e8c8883c71bb75c3040f))
    - Cleanup hooks ([`9234c37`](https://github.com/rootCircle/bgit/commit/9234c37f487cd47bf93e0d04d3535ff0a2ccf355))
    - Chore(ci): remove unused GPG signing from release workflow The GPG signing of the RELEASES.txt file was removed from the release workflow. This step is no longer necessary as the release assets are signed individually. ([`a48ca23`](https://github.com/rootCircle/bgit/commit/a48ca2362652d1bd6b35875b2a48b1564392fe52))
    - Release bgit v0.3.3 ([`424d5e2`](https://github.com/rootCircle/bgit/commit/424d5e2806b3593285501a2753de19b2f4cff6f5))
    - Add HOOKS.md and improve hook execution ([`377aeb5`](https://github.com/rootCircle/bgit/commit/377aeb54882c7690badd28a42f3493b54e9c9eda))
    - Improve error messages and tips for adding remotes ([`86564ed`](https://github.com/rootCircle/bgit/commit/86564edfc5eb4d52b9ebbee5e7f41fdb8d288cfd))
    - Update installation instructions and add install scripts ([`178e216`](https://github.com/rootCircle/bgit/commit/178e216e48ecab71464b2750863a90a0ea1b6f4c))
    - Add cross compilation and artifact packaging for more targets ([`e39ff22`](https://github.com/rootCircle/bgit/commit/e39ff22ae46c44be621b1df0a594539c0dcd9f35))
</details>

## v0.3.2 (2025-08-13)

<csr-id-2df5ff4a7841a13371a0341c30205e042d0a8fa0/>
<csr-id-4129d48f5a7d491271b7866d5e8f015788f5f709/>
<csr-id-5ff6992d9c07e228f410929084a2218ae83e1058/>

### Other

 - <csr-id-2df5ff4a7841a13371a0341c30205e042d0a8fa0/> add release workflow and update Makefile
   - Add GitHub Actions workflow for automated releases on tag pushes.
   - Workflow includes building artifacts for different OS, verifying tag
     matches Cargo.toml, and creating a GitHub release with assets.
   - Update Makefile to use `--no-changelog-github-release` with `cargo
     smart-release` to prevent duplicate changelog entries.
 - <csr-id-4129d48f5a7d491271b7866d5e8f015788f5f709/> clippy fixes

### Refactor

 - <csr-id-5ff6992d9c07e228f410929084a2218ae83e1058/> Extract auth callbacks to shared module
   - Moved git authentication logic from `GitPull::setup_auth_callbacks` to a new shared function `setup_auth_callbacks` in `src/auth/git_auth.rs`.
   - Updated `GitPull::create_fetch_options` to use the new shared function.
   - This promotes code reuse and centralizes authentication handling.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release bgit v0.3.2 ([`611a386`](https://github.com/rootCircle/bgit/commit/611a38603f3288c24e2df626c39316d4e6cf67de))
    - Extract auth callbacks to shared module ([`5ff6992`](https://github.com/rootCircle/bgit/commit/5ff6992d9c07e228f410929084a2218ae83e1058))
    - Add release workflow and update Makefile ([`2df5ff4`](https://github.com/rootCircle/bgit/commit/2df5ff4a7841a13371a0341c30205e042d0a8fa0))
    - Clippy fixes ([`4129d48`](https://github.com/rootCircle/bgit/commit/4129d48f5a7d491271b7866d5e8f015788f5f709))
</details>

## v0.3.1 (2025-08-13)

<csr-id-e0cd3717282b40f02d92fee7b594b62be739bc3e/>
<csr-id-8eae1be826e4fa704bdfdd4a054a15ef208e9c98/>
<csr-id-4435aba169a68cd271b6e59af7007d8cc9733526/>
<csr-id-261ccfe5b29066a98e804b9f4f189e5bba603a82/>
<csr-id-7fa30f5df8cef0f28fca0d664d23aee9cc93b3e6/>
<csr-id-a8b37f7fd09dc39ef45c926dcca23ac6e716d64c/>
<csr-id-2d435c16783331a1780048df8a23ec5e2793143f/>
<csr-id-66531db9854a3d28d694177230e24d4943022c42/>
<csr-id-02492a4adcef732063d5174d7ba53e0b8ad6a4bc/>

### Chore

 - <csr-id-e0cd3717282b40f02d92fee7b594b62be739bc3e/> git pull fix
 - <csr-id-8eae1be826e4fa704bdfdd4a054a15ef208e9c98/> move few helper method to impl
 - <csr-id-4435aba169a68cd271b6e59af7007d8cc9733526/> unimplemented error on other subcommands
 - <csr-id-261ccfe5b29066a98e804b9f4f189e5bba603a82/> update LICENSE owners & lint

### Documentation

 - <csr-id-c4ce388fb4278ede5d8720f75540f0a7b6a82132/> add library links
 - <csr-id-3dbf649e29f1457d2c0ee50394b07fc390d5eb02/> fix overide sole contributor values

### New Features

<csr-id-18ab7a43e51164a4b481260b570e040af5f32d76/>
<csr-id-886e05e1fb9ce299d9ee259696dce24ced9d5ac0/>
<csr-id-562f83824897e4dc5d768b74e215172ef83545bc/>
<csr-id-dcd50c9b763dd66a80393f2c81d8951a108977c5/>
<csr-id-ecbf6af27a7eb763437b161c4041db44f8bfd0c8/>
<csr-id-c74a22ca4c41c62a3d573084494201d597905b14/>

 - <csr-id-71760ebb6c8e6903f0c353938f0cef04ba590c33/> Enhance SSH Key Handling and Git Push Functionality
   - Improved SSH key handling by adding the `home` crate to determine the user's home directory and fixing ssh-agent lifecycle issues.

### Bug Fixes

 - <csr-id-af08bbbed1f079c6ea2d9f7556dc5f0dbc2bc601/> fix class of bugs which occurs on empty repo with no commit (repo.head())
   added rule of remotes exists
 - <csr-id-1fc4d3cead93733251952edaa09a0cc56004f3e4/> fix clippy issue
 - <csr-id-1684ada0186c5d237fe7d0b41dc4904c8624de53/> remove unnecessary check of unstaged files
 - <csr-id-e543f195bdf7d745def11fe52094281ee1b1756b/> resolve nitpicks
 - <csr-id-2dea085e9d52ab736b1739f6a41ee26fd570d3fa/> replace force with force_with_lease
 - <csr-id-3670d97992f8237be5744d89251a0ad66a1351a7/> strip .git suffix from cloned repository folder names

### Other

 - <csr-id-7fa30f5df8cef0f28fca0d664d23aee9cc93b3e6/> cargo sort
 - <csr-id-a8b37f7fd09dc39ef45c926dcca23ac6e716d64c/> upgrade package
 - <csr-id-2d435c16783331a1780048df8a23ec5e2793143f/> remove codecov integration
 - <csr-id-66531db9854a3d28d694177230e24d4943022c42/> Improve large file detection and LFS suggestion
   This commit fixes the following issues:
   
   - Fixes a bug where capture group 2 would error if it didn't exist.
   - Changes the NoLargeFile rule level to "error" to ensure that large files are not committed.
   - Enhances large file detection to consider both individual file size and total size of staged/modified files.
   - Improves the suggestion messages for using Git LFS, including instructions for tracking files by extension or specific files.
   - Adds automatic LFS tracking by extension to .gitattributes.

### Refactor

 - <csr-id-02492a4adcef732063d5174d7ba53e0b8ad6a4bc/> simplify error handling in GitStatus methods

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 27 commits contributed to the release over the course of 63 calendar days.
 - 64 days passed between releases.
 - 24 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#7](https://github.com/rootCircle/bgit/issues/7)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#7](https://github.com/rootCircle/bgit/issues/7)**
    - Enhance SSH Key Handling and Git Push Functionality ([`71760eb`](https://github.com/rootCircle/bgit/commit/71760ebb6c8e6903f0c353938f0cef04ba590c33))
 * **Uncategorized**
    - Release bgit v0.3.1 ([`bad33a9`](https://github.com/rootCircle/bgit/commit/bad33a931050b0e48b8050ef84c4ad8479d62515))
    - Git pull fix ([`e0cd371`](https://github.com/rootCircle/bgit/commit/e0cd3717282b40f02d92fee7b594b62be739bc3e))
    - Improve rule efficiency and add Rust 1.88 support ([`18ab7a4`](https://github.com/rootCircle/bgit/commit/18ab7a43e51164a4b481260b570e040af5f32d76))
    - Refactor to rig-core and validate commit message ([`886e05e`](https://github.com/rootCircle/bgit/commit/886e05e1fb9ce299d9ee259696dce24ced9d5ac0))
    - Cargo sort ([`7fa30f5`](https://github.com/rootCircle/bgit/commit/7fa30f5df8cef0f28fca0d664d23aee9cc93b3e6))
    - Simplify error handling in GitStatus methods ([`02492a4`](https://github.com/rootCircle/bgit/commit/02492a4adcef732063d5174d7ba53e0b8ad6a4bc))
    - Move few helper method to impl ([`8eae1be`](https://github.com/rootCircle/bgit/commit/8eae1be826e4fa704bdfdd4a054a15ef208e9c98))
    - Refactor error handling in Git event modules ([`9b9786e`](https://github.com/rootCircle/bgit/commit/9b9786e4c8f29c59af10c7658feec682c53d54f7))
    - IT setup ([`562f838`](https://github.com/rootCircle/bgit/commit/562f83824897e4dc5d768b74e215172ef83545bc))
    - Fix class of bugs which occurs on empty repo with no commit (repo.head()) ([`af08bbb`](https://github.com/rootCircle/bgit/commit/af08bbbed1f079c6ea2d9f7556dc5f0dbc2bc601))
    - Fix clippy issue ([`1fc4d3c`](https://github.com/rootCircle/bgit/commit/1fc4d3cead93733251952edaa09a0cc56004f3e4))
    - Upgrade package ([`a8b37f7`](https://github.com/rootCircle/bgit/commit/a8b37f7fd09dc39ef45c926dcca23ac6e716d64c))
    - Unimplemented error on other subcommands ([`4435aba`](https://github.com/rootCircle/bgit/commit/4435aba169a68cd271b6e59af7007d8cc9733526))
    - Update LICENSE owners & lint ([`261ccfe`](https://github.com/rootCircle/bgit/commit/261ccfe5b29066a98e804b9f4f189e5bba603a82))
    - Remove codecov integration ([`2d435c1`](https://github.com/rootCircle/bgit/commit/2d435c16783331a1780048df8a23ec5e2793143f))
    - Remove unnecessary check of unstaged files ([`1684ada`](https://github.com/rootCircle/bgit/commit/1684ada0186c5d237fe7d0b41dc4904c8624de53))
    - Resolve nitpicks ([`e543f19`](https://github.com/rootCircle/bgit/commit/e543f195bdf7d745def11fe52094281ee1b1756b))
    - Replace force with force_with_lease ([`2dea085`](https://github.com/rootCircle/bgit/commit/2dea085e9d52ab736b1739f6a41ee26fd570d3fa))
    - Strip .git suffix from cloned repository folder names ([`3670d97`](https://github.com/rootCircle/bgit/commit/3670d97992f8237be5744d89251a0ad66a1351a7))
    - Add library links ([`c4ce388`](https://github.com/rootCircle/bgit/commit/c4ce388fb4278ede5d8720f75540f0a7b6a82132))
    - Refactor file size checks, add install target ([`dcd50c9`](https://github.com/rootCircle/bgit/commit/dcd50c9b763dd66a80393f2c81d8951a108977c5))
    - Improve large file detection and LFS suggestion ([`66531db`](https://github.com/rootCircle/bgit/commit/66531db9854a3d28d694177230e24d4943022c42))
    - Add pre-commit hook and file size checks ([`ecbf6af`](https://github.com/rootCircle/bgit/commit/ecbf6af27a7eb763437b161c4041db44f8bfd0c8))
    - Use constants for default file size limits ([`c74a22c`](https://github.com/rootCircle/bgit/commit/c74a22ca4c41c62a3d573084494201d597905b14))
    - Fix overide sole contributor values ([`3dbf649`](https://github.com/rootCircle/bgit/commit/3dbf649e29f1457d2c0ee50394b07fc390d5eb02))
    - Fix broken image in ARCHITECTURE.md ([`750b25f`](https://github.com/rootCircle/bgit/commit/750b25f515e47f27ba0093b3cb157ff688eac4ec))
</details>

## v0.3.0 (2025-06-09)

<csr-id-27967a17ab68bf3e07088c1fab6de4112f890f1e/>
<csr-id-32430e09cd5449b53cdda54c04435ca10af88b78/>
<csr-id-8cb9729c58bf55edb4e4a9336812fd6590259b2c/>
<csr-id-66eba1cb67da5875d382e94b79a874e283a200e2/>
<csr-id-22a7a4f9c35048fabe48b23f2505dc1993daf177/>
<csr-id-2629db5bcd9dd8c9f8571f00841fbc632c19ede5/>
<csr-id-eea360d35a74e124e56cacaaa3788721dd83508f/>
<csr-id-cf15617fc71e1fe3ed0f8cca8bf7b3f0d948d377/>
<csr-id-36d716c5a95ef05a9ffae08c73dbe4b825868b75/>
<csr-id-69e547e97cd69bbe633153db32f1ac0a1c5ffd68/>
<csr-id-717f4c431234d0c36332f54951c0568c21af8c9a/>
<csr-id-7aa713ebf8454c70e3c7e8e5d3743833490ce74d/>
<csr-id-82d6a655f75a6639f144cf8b9cb2cea12cf8c9c4/>
<csr-id-f3bd90f4c2664d192d4ff48ce9141fd02adae192/>
<csr-id-09089edc9d092e3daeb83b7a07cd387961d51bab/>
<csr-id-ea32487b5a552fbaf4ab3aaa19fcbd4aaf84c308/>

### Chore

 - <csr-id-27967a17ab68bf3e07088c1fab6de4112f890f1e/> Update project metadata and improve function interfaces
   - Updated authors and repository URL in Cargo.toml
   - Enhanced project description and categories in Cargo.toml
   - Updated dependencies to their latest versions in Cargo.toml
   - Renamed `with_stash_message` to `set_stash_message` in GitBranch for clarity
   - Updated `with_rebase` method in GitPull to take ownership of self
   - Refactored `with_mode` methods in GitRestore, RestoreChanges, and MoveChanges to take ownership of self
   - Cleaned up whitespace in main.rs
   - Updated action implementations to use new method signatures for improved clarity
 - <csr-id-32430e09cd5449b53cdda54c04435ca10af88b78/> authenticated git clone support
 - <csr-id-8cb9729c58bf55edb4e4a9336812fd6590259b2c/> remove dead/redundant code and improve code semantics
 - <csr-id-66eba1cb67da5875d382e94b79a874e283a200e2/> remove unnecessary debug info prints
 - <csr-id-22a7a4f9c35048fabe48b23f2505dc1993daf177/> switch tokio to latest stable version
 - <csr-id-2629db5bcd9dd8c9f8571f00841fbc632c19ede5/> prune dead_code
 - <csr-id-eea360d35a74e124e56cacaaa3788721dd83508f/> clean_up
 - <csr-id-cf15617fc71e1fe3ed0f8cca8bf7b3f0d948d377/> system aware rule fix

### Documentation

 - <csr-id-a0316f9266a7d7a5b02a29b6143337544095b7c0/> Usage instructions
 - <csr-id-f222752d2c0ea3ecd9a978593ffe959ad22b4cb8/> add meta links
 - <csr-id-48dd57a8d2a15fddcd076ba1413ee529633bfcc3/> created markdown rule for multiple RULES

### New Features

<csr-id-c0eb8ca7fb8e9c900cdb3990b941fc855e5cb681/>
<csr-id-6e24afcd67583b4cc54b83d4f88786cf892e1e94/>
<csr-id-420c4a287b5ab93ce5766a65cf993b852291bc6c/>
<csr-id-6691c85efd0353b4f899661096c837297f8cb97d/>
<csr-id-4289ea4ffa5b3b83f0c08f112bb38144d58bda62/>
<csr-id-2dd132bd676e1a39f585b4d210addf61cd50c18f/>
<csr-id-955b69109c3684735a7964ad178f17ca849ad438/>
<csr-id-599aae8e9552c7d8720851bc135d7cc1e50e42fc/>
<csr-id-280a08ac17d2e7385c3b0b800218934add36bc4d/>
<csr-id-e08d075820f025c8a20fba78ac0ca9c65b6b3e92/>
<csr-id-64de7d6d4b8f630e391f9a41dd16c51a2dbd6ed3/>
<csr-id-45747918d39402f83f798366f1cad17f5faf8ccb/>
<csr-id-2e6460d940137dc4d096de39bf4facd6687cf24c/>

 - <csr-id-d5f5f4da14deb4146e5ac2914ac7322fe54fb122/> add multi_select
 - <csr-id-a3f1cad2c0d4aefdcdb219c616d36a65a52547ae/> add git configuration management and workflow flag constants
 - <csr-id-e730fac235e10bf49bd8b8a56e409837954be134/> update workflow rules to improve error handling and integrate rule configuration
 - <csr-id-6bcb5261c08d161418e3c1e088c7c7d790a3aea5/> add configuration management and update command functions to utilize BGitConfig
 - <csr-id-8ecb7541fa1ceb667a038a462e703749115583d4/> implement conventional commit message rule
   This commit introduces a new rule that validates commit messages
   against the Conventional Commits specification. It also integrates
   this rule into the AI-generated commit message and ask human commit
   message workflows.
 - <csr-id-9d8849008a82ac9b8e61fa3b83fa95f0d5f5cdf5/> Add file size check and Git LFS integration
   This commit introduces a new rule, `NoLargeFile`, to check for large files not tracked by Git LFS.  It also adds the existing secret and repo size rules to the `git_add` event, and removes redundant rule additions in workflow steps, consolidating them in the event definitions.
 - <csr-id-3cbab2545894f5b8a92aabfc96db06303ab98a73/> Add rule to prevent staging secret files
   This commit introduces a new rule, `NoSecretFilesStaged`, to prevent the staging of files that might contain secrets. The rule checks staged and modified files against a list of regular expressions to identify potential secret files. If any are found, it raises an error.
   
   The commit also adds the new rule to `GitAdd` and `GitCommit` events.
   
   Also the commit fixes a typo.
 - <csr-id-a00ef5674312e9bf50a4a0c1c9c6e21ca55d43ac/> Add Git user.name and user.email setup rule
 - <csr-id-4dec86a563e54456dbf84f5e0879822b1e56a881/> Improve branch creation and validation
   - Preserve staging state when popping stash

### Bug Fixes

 - <csr-id-0580af6234e2720148f1269232a1d116a4247046/> return error if restore mode is not specified
 - <csr-id-fafcf879bb9e79785072d0e95b1dac8944b330ae/> Remove accidental text from README
 - <csr-id-10540cb1db8d243bb23ea18a70324c5f651e8019/> bug with ai message taking diff not staged diff
 - <csr-id-086e73e9cdfb064f67f2da2ae13d7097c0e95475/> transient error message when raw_execute in event return false
 - <csr-id-25d57d69ed5dbaae0a025d674de8b5e3ef96871a/> clippy warnings and fmt errors
 - <csr-id-3588981297dcdbb641ff39511e8ba2461216a852/> replace repo.open with repo.discover
 - <csr-id-e31b1a21ba418a5b5a90573fa62be5bf2901c980/> segregate prompt_step and events from events/git_add
 - <csr-id-79b93526cad9587d8ceb86c75fd5691e2b1bd52f/> replace use of command spawn
 - <csr-id-188679258bc67465c65b4433b1b5f865511d3909/> robust secret checking
 - <csr-id-2c0b491cf3121930fcb527a2b52833f0cdeb36f1/> auto-fix
 - <csr-id-c6782b0bd59911c552b1c74935b5653512e26e1e/> events/push/pull

### Other

 - <csr-id-36d716c5a95ef05a9ffae08c73dbe4b825868b75/> bump version to 0.3.0
 - <csr-id-69e547e97cd69bbe633153db32f1ac0a1c5ffd68/> upgrade package
 - <csr-id-717f4c431234d0c36332f54951c0568c21af8c9a/> Improve author email check in contributor action
   This commit refactors the author email check in the `IsSoleContributor` action.
   It now uses `author_emails.contains(&current_author_email)` for more efficient
   email matching and improves code readability. Also moves flags module load.
 - <csr-id-7aa713ebf8454c70e3c7e8e5d3743833490ce74d/> cargo fmt
 - <csr-id-82d6a655f75a6639f144cf8b9cb2cea12cf8c9c4/> cargo fmt
 - <csr-id-f3bd90f4c2664d192d4ff48ce9141fd02adae192/> cargo fmt
 - <csr-id-09089edc9d092e3daeb83b7a07cd387961d51bab/> fix

### Refactor

 - <csr-id-ea32487b5a552fbaf4ab3aaa19fcbd4aaf84c308/> Refactor workflow execution to incorporate workflow rules and step flags
   - Updated `default_cmd_workflow` to retrieve workflow rules and steps from configuration.
   - Introduced `get_workflow_rules` and `get_workflow_steps` methods in `BGitConfig` for better access to workflow configurations.
   - Modified `WorkflowQueue` to accept workflow rules and step flags during execution.
   - Enhanced `ActionStep` and `PromptStep` traits to include parameters for step configuration flags and workflow rules.
   - Updated all action and prompt implementations to accommodate the new execute signature.
   - Added tests to verify the correct retrieval and usage of workflow rules and step flags.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 62 commits contributed to the release over the course of 72 calendar days.
 - 94 days passed between releases.
 - 52 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release bgit v0.3.0 ([`6ddca85`](https://github.com/rootCircle/bgit/commit/6ddca8542a4fe0a34492fc1659e613889f5ce802))
    - Bump version to 0.3.0 ([`36d716c`](https://github.com/rootCircle/bgit/commit/36d716c5a95ef05a9ffae08c73dbe4b825868b75))
    - Upgrade package ([`69e547e`](https://github.com/rootCircle/bgit/commit/69e547e97cd69bbe633153db32f1ac0a1c5ffd68))
    - Return error if restore mode is not specified ([`0580af6`](https://github.com/rootCircle/bgit/commit/0580af6234e2720148f1269232a1d116a4247046))
    - Add multi_select ([`d5f5f4d`](https://github.com/rootCircle/bgit/commit/d5f5f4da14deb4146e5ac2914ac7322fe54fb122))
    - Improve author email check in contributor action ([`717f4c4`](https://github.com/rootCircle/bgit/commit/717f4c431234d0c36332f54951c0568c21af8c9a))
    - Add git configuration management and workflow flag constants ([`a3f1cad`](https://github.com/rootCircle/bgit/commit/a3f1cad2c0d4aefdcdb219c616d36a65a52547ae))
    - Update workflow rules to improve error handling and integrate rule configuration ([`e730fac`](https://github.com/rootCircle/bgit/commit/e730fac235e10bf49bd8b8a56e409837954be134))
    - Refactor workflow execution to incorporate workflow rules and step flags ([`ea32487`](https://github.com/rootCircle/bgit/commit/ea32487b5a552fbaf4ab3aaa19fcbd4aaf84c308))
    - Cargo fmt ([`7aa713e`](https://github.com/rootCircle/bgit/commit/7aa713ebf8454c70e3c7e8e5d3743833490ce74d))
    - Add configuration management and update command functions to utilize BGitConfig ([`6bcb526`](https://github.com/rootCircle/bgit/commit/6bcb5261c08d161418e3c1e088c7c7d790a3aea5))
    - Implement conventional commit message rule ([`8ecb754`](https://github.com/rootCircle/bgit/commit/8ecb7541fa1ceb667a038a462e703749115583d4))
    - Add file size check and Git LFS integration ([`9d88490`](https://github.com/rootCircle/bgit/commit/9d8849008a82ac9b8e61fa3b83fa95f0d5f5cdf5))
    - Add rule to prevent staging secret files ([`3cbab25`](https://github.com/rootCircle/bgit/commit/3cbab2545894f5b8a92aabfc96db06303ab98a73))
    - Add Git user.name and user.email setup rule ([`a00ef56`](https://github.com/rootCircle/bgit/commit/a00ef5674312e9bf50a4a0c1c9c6e21ca55d43ac))
    - Improve branch creation and validation ([`4dec86a`](https://github.com/rootCircle/bgit/commit/4dec86a563e54456dbf84f5e0879822b1e56a881))
    - Remove accidental text from README ([`fafcf87`](https://github.com/rootCircle/bgit/commit/fafcf879bb9e79785072d0e95b1dac8944b330ae))
    - Improve hook executor logging ([`c0eb8ca`](https://github.com/rootCircle/bgit/commit/c0eb8ca7fb8e9c900cdb3990b941fc855e5cb681))
    - Add post-git clone hook and update README ([`6e24afc`](https://github.com/rootCircle/bgit/commit/6e24afcd67583b4cc54b83d4f88786cf892e1e94))
    - Add pre-commit hook and improve prompts ([`420c4a2`](https://github.com/rootCircle/bgit/commit/420c4a287b5ab93ce5766a65cf993b852291bc6c))
    - Update project metadata and improve function interfaces ([`27967a1`](https://github.com/rootCircle/bgit/commit/27967a17ab68bf3e07088c1fab6de4112f890f1e))
    - Add verbosity flag for logging ([`6691c85`](https://github.com/rootCircle/bgit/commit/6691c85efd0353b4f899661096c837297f8cb97d))
    - Add logging for debugging ([`4289ea4`](https://github.com/rootCircle/bgit/commit/4289ea4ffa5b3b83f0c08f112bb38144d58bda62))
    - Cargo fmt ([`82d6a65`](https://github.com/rootCircle/bgit/commit/82d6a655f75a6639f144cf8b9cb2cea12cf8c9c4))
    - Authenticated git clone support ([`32430e0`](https://github.com/rootCircle/bgit/commit/32430e09cd5449b53cdda54c04435ca10af88b78))
    - Bug with ai message taking diff not staged diff ([`10540cb`](https://github.com/rootCircle/bgit/commit/10540cb1db8d243bb23ea18a70324c5f651e8019))
    - Propogate result status from raw_execute to execute ([`800cdcd`](https://github.com/rootCircle/bgit/commit/800cdcdfc555e33fd704e8796be4cffcfd1586c8))
    - Transient error message when raw_execute in event return false ([`086e73e`](https://github.com/rootCircle/bgit/commit/086e73e9cdfb064f67f2da2ae13d7097c0e95475))
    - Remove dead/redundant code and improve code semantics ([`8cb9729`](https://github.com/rootCircle/bgit/commit/8cb9729c58bf55edb4e4a9336812fd6590259b2c))
    - Clippy warnings and fmt errors ([`25d57d6`](https://github.com/rootCircle/bgit/commit/25d57d69ed5dbaae0a025d674de8b5e3ef96871a))
    - Remove unnecessary debug info prints ([`66eba1c`](https://github.com/rootCircle/bgit/commit/66eba1cb67da5875d382e94b79a874e283a200e2))
    - Fmt ([`74a7bfb`](https://github.com/rootCircle/bgit/commit/74a7bfbe273dd094dfe7efe689c1464aba703694))
    - Replace repo.open with repo.discover ([`3588981`](https://github.com/rootCircle/bgit/commit/3588981297dcdbb641ff39511e8ba2461216a852))
    - Clean_up ([`f8c238a`](https://github.com/rootCircle/bgit/commit/f8c238a241c766abe7ccb3723a9a757b86a6c0fe))
    - Segregate prompt_step and events from events/git_add ([`e31b1a2`](https://github.com/rootCircle/bgit/commit/e31b1a21ba418a5b5a90573fa62be5bf2901c980))
    - Switch tokio to latest stable version ([`22a7a4f`](https://github.com/rootCircle/bgit/commit/22a7a4f9c35048fabe48b23f2505dc1993daf177))
    - Prune dead_code ([`2629db5`](https://github.com/rootCircle/bgit/commit/2629db5bcd9dd8c9f8571f00841fbc632c19ede5))
    - Replace use of command spawn ([`79b9352`](https://github.com/rootCircle/bgit/commit/79b93526cad9587d8ceb86c75fd5691e2b1bd52f))
    - Clean_up ([`eea360d`](https://github.com/rootCircle/bgit/commit/eea360d35a74e124e56cacaaa3788721dd83508f))
    - Robust secret checking ([`1886792`](https://github.com/rootCircle/bgit/commit/188679258bc67465c65b4433b1b5f865511d3909))
    - Auto-fix ([`2c0b491`](https://github.com/rootCircle/bgit/commit/2c0b491cf3121930fcb527a2b52833f0cdeb36f1))
    - Add no_secrets and repo_too_big ([`2dd132b`](https://github.com/rootCircle/bgit/commit/2dd132bd676e1a39f585b4d210addf61cd50c18f))
    - WF complete-all modules of workflow tested and working ([`955b691`](https://github.com/rootCircle/bgit/commit/955b69109c3684735a7964ad178f17ca849ad438))
    - Actions and prompt WF complete till askcommit ([`599aae8`](https://github.com/rootCircle/bgit/commit/599aae8e9552c7d8720851bc135d7cc1e50e42fc))
    - Events/push/pull ([`c6782b0`](https://github.com/rootCircle/bgit/commit/c6782b0bd59911c552b1c74935b5653512e26e1e))
    - Complete push/pull WF. Known error at events/git_status/has_unpushed_commits ([`280a08a`](https://github.com/rootCircle/bgit/commit/280a08ac17d2e7385c3b0b800218934add36bc4d))
    - Add actions and prompt WF till restore changes ([`e08d075`](https://github.com/rootCircle/bgit/commit/e08d075820f025c8a20fba78ac0ca9c65b6b3e92))
    - Add actions and prompts WF till add to staging ([`64de7d6`](https://github.com/rootCircle/bgit/commit/64de7d6d4b8f630e391f9a41dd16c51a2dbd6ed3))
    - Add actions and prompt WF till stash events ([`4574791`](https://github.com/rootCircle/bgit/commit/45747918d39402f83f798366f1cad17f5faf8ccb))
    - Usage instructions ([`a0316f9`](https://github.com/rootCircle/bgit/commit/a0316f9266a7d7a5b02a29b6143337544095b7c0))
    - Cargo fmt ([`f3bd90f`](https://github.com/rootCircle/bgit/commit/f3bd90f4c2664d192d4ff48ce9141fd02adae192))
    - System aware rule fix ([`cf15617`](https://github.com/rootCircle/bgit/commit/cf15617fc71e1fe3ed0f8cca8bf7b3f0d948d377))
    - Add meta links ([`f222752`](https://github.com/rootCircle/bgit/commit/f222752d2c0ea3ecd9a978593ffe959ad22b4cb8))
    - Merge pull request #13 from Him7n/main ([`a2e4022`](https://github.com/rootCircle/bgit/commit/a2e4022d2a52370db87773b9b4ab2f745e3b48a8))
    - Created markdown rule for multiple RULES ([`48dd57a`](https://github.com/rootCircle/bgit/commit/48dd57a8d2a15fddcd076ba1413ee529633bfcc3))
    - Merge pull request #11 from Him7n/main ([`d8fc16d`](https://github.com/rootCircle/bgit/commit/d8fc16d7dc62ce5e44d0399453f067715c9c0166))
    - Merge branch 'Gyan172004:main' into main ([`d899a75`](https://github.com/rootCircle/bgit/commit/d899a756ee829ef5c7dc12f03f89afad5ba20f6b))
    - Added RuleLevel enforcement to template and introduced "Possible Fixes" section ([`66f90c5`](https://github.com/rootCircle/bgit/commit/66f90c5d87bf6bfc9e96cf2d35766c832884daf0))
    - Merge pull request #10 from Him7n/main ([`1e1307d`](https://github.com/rootCircle/bgit/commit/1e1307d0f68c4c5341f95628f30024ea47a1c34c))
    - Add RFC-style Git Rule Specification template ([`01592bc`](https://github.com/rootCircle/bgit/commit/01592bc7a49a05a7ec66d09b0aa98764972852f8))
    - Fix ([`09089ed`](https://github.com/rootCircle/bgit/commit/09089edc9d092e3daeb83b7a07cd387961d51bab))
    - Improve Task enum matching logic in PartialEq implementation ([`2e6460d`](https://github.com/rootCircle/bgit/commit/2e6460d940137dc4d096de39bf4facd6687cf24c))
</details>

## v0.2.1 (2025-03-07)

<csr-id-8cd7160abfaa33da9dd2db1b58a7ed4eaf3a4db6/>
<csr-id-1c5ac77681ee1f446cd5af527e151d456cf69838/>
<csr-id-b7a64a7b7a2973a4923b7a3abad6656c60656c76/>
<csr-id-701e5dfe2b1aeb70a58c45d1dad705b7a2a377d7/>
<csr-id-a8528e3deb827643cd4fda69245fb86218531961/>
<csr-id-8cff501b2278c3407ebaeb582f2e0abda5f8e27d/>
<csr-id-fb5ee8d35b9c955d70973cd3ce330767ee1d10de/>
<csr-id-e95d8c35b1c924de1feaf011872781f711f2a6ad/>
<csr-id-4bb574f5a0a2cd318ff7b18286c6856cd56c4aa0/>
<csr-id-3685f3c011840588ff892c7c57259dd62c6f2477/>
<csr-id-c555361aa97d43c1b86840401c7ebe883c544dfe/>
<csr-id-a47e1126912a9e84a7dd48e8ad38386ed6c5057e/>
<csr-id-f8bdce230d3edc08a2944c335e648e2835eabd48/>
<csr-id-ea17e150e9839dede10ed3f568d67f22bc8b7416/>
<csr-id-376da8460a8e0bd4c45d9cbc582365a959e399a3/>
<csr-id-fb00861d9ca0b9498f2efb87543735a6ce3849c3/>
<csr-id-8045ea32b7a5e4b259934541b72ad9d285d84bda/>
<csr-id-dd4c718df7b0f27c9498cfa481866c47dbda18bd/>

### Chore

 - <csr-id-8cd7160abfaa33da9dd2db1b58a7ed4eaf3a4db6/> release script
 - <csr-id-1c5ac77681ee1f446cd5af527e151d456cf69838/> add error types
 - <csr-id-b7a64a7b7a2973a4923b7a3abad6656c60656c76/> add error types
 - <csr-id-701e5dfe2b1aeb70a58c45d1dad705b7a2a377d7/> fix tags
 - <csr-id-a8528e3deb827643cd4fda69245fb86218531961/> migrate to rust 2024 and some ci
 - <csr-id-8cff501b2278c3407ebaeb582f2e0abda5f8e27d/> fix typo
 - <csr-id-fb5ee8d35b9c955d70973cd3ce330767ee1d10de/> fix os parity checks
 - <csr-id-e95d8c35b1c924de1feaf011872781f711f2a6ad/> renamed commit hook name format
 - <csr-id-4bb574f5a0a2cd318ff7b18286c6856cd56c4aa0/> add sample cmd usages and deps
 - <csr-id-3685f3c011840588ff892c7c57259dd62c6f2477/> add rules

### Documentation

 - <csr-id-b6a4b5561581c9d96e4ff3794c083d7cccc4356e/> windows hook_executor is implemented and fairly stable now
 - <csr-id-2cb356903243c6a8aee7e7d67f78930fc43e41e5/> add shields badge
 - <csr-id-e79bafc4122361dbfda32b87c40f415397a94413/> add arch docs
 - <csr-id-fc72b4b6acf1914f847cfbd63653b314240eb338/> add workflows

### New Features

 - <csr-id-f39eacb0f6235a8be0ea4eafea7cad363923f13a/> add update_cwd_path method to GitClone and InitGitRepo for setting current working directory
 - <csr-id-dd76e5154ec3681aba00f23d945802ee5197305c/> add new error types and update module visibility
 - <csr-id-b93abfc35d659087988f1a1bbad73aa265dc4cda/> add new error types and update module visibility
 - <csr-id-daa823ffdb57d1f71c5c889eb9e7a53a3a25dbbb/> implement full git add logic using libgit2
   This commit introduces a complete implementation of git add functionality in the GitAdd event. Instead of using a placeholder command, the new code leverages libgit2 (via the git2 crate) to open the repository, retrieve the index, add all files recursively, and write the index to disk. Detailed error handling is provided via BGitError.
 - <csr-id-dd2a5dcf2a2938686ca938184e4df79812161fe3/> improved ui for cli
 - <csr-id-82c701afb5230e3e759875103806c9f680657aec/> common action store implemented
 - <csr-id-3b8196f3d7008f404af5baa943bd867c8e25098b/> Heap for error type and refactor and hook executor
 - <csr-id-ff7a9b546891852208c01596a0d10ba387a6eddc/> code structure
 - <csr-id-ec2b4318007176ab9fcc8673c37e15e07ad90c14/> add MIT license
 - <csr-id-22aa5c046d2d68e834a88136526bee58658637df/> initial commit

### Bug Fixes

 - <csr-id-add40ef35161ba0efa46f5b363dd6c560f0279fb/> fix windows ci
 - <csr-id-1b7e460a2394bea0c9e69495513a2f3e326067fe/> fix windows ci
 - <csr-id-ec365a0a575fc19ace3ce458041f3c916260692e/> fix prompt dialog mangled into progress bar

### Other

 - <csr-id-c555361aa97d43c1b86840401c7ebe883c544dfe/> windows runners as well
 - <csr-id-a47e1126912a9e84a7dd48e8ad38386ed6c5057e/> windows runners as well
 - <csr-id-f8bdce230d3edc08a2944c335e648e2835eabd48/> code cov check
 - <csr-id-ea17e150e9839dede10ed3f568d67f22bc8b7416/> fix
 - <csr-id-376da8460a8e0bd4c45d9cbc582365a959e399a3/> metadata changes
 - <csr-id-fb00861d9ca0b9498f2efb87543735a6ce3849c3/> add action scripts for test and build

### Refactor

 - <csr-id-8045ea32b7a5e4b259934541b72ad9d285d84bda/> remove old InitGitRepo action and replace with new prompt implementation
 - <csr-id-dd4c718df7b0f27c9498cfa481866c47dbda18bd/> remove name, description from trait definition for new

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 53 commits contributed to the release over the course of 319 calendar days.
 - 35 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release bgit v0.2.1 ([`adbe0b0`](https://github.com/rootCircle/bgit/commit/adbe0b01348f45431df35706691e8bc7097f9282))
    - Release script ([`8cd7160`](https://github.com/rootCircle/bgit/commit/8cd7160abfaa33da9dd2db1b58a7ed4eaf3a4db6))
    - Release bgit v0.2.0 ([`3f1d285`](https://github.com/rootCircle/bgit/commit/3f1d2853bc8f115b21bfe3634c6dd64afe574eeb))
    - Windows hook_executor is implemented and fairly stable now ([`b6a4b55`](https://github.com/rootCircle/bgit/commit/b6a4b5561581c9d96e4ff3794c083d7cccc4356e))
    - Add update_cwd_path method to GitClone and InitGitRepo for setting current working directory ([`f39eacb`](https://github.com/rootCircle/bgit/commit/f39eacb0f6235a8be0ea4eafea7cad363923f13a))
    - Remove old InitGitRepo action and replace with new prompt implementation ([`8045ea3`](https://github.com/rootCircle/bgit/commit/8045ea32b7a5e4b259934541b72ad9d285d84bda))
    - Remove name, description from trait definition for new ([`dd4c718`](https://github.com/rootCircle/bgit/commit/dd4c718df7b0f27c9498cfa481866c47dbda18bd))
    - Implemented tasks ask_to_init_clone_git , init_git_repo , ask_to_clone_git_repo ; events git clone and git init and some minor refactoring ([`b937519`](https://github.com/rootCircle/bgit/commit/b937519a495686e00cb853c5acb38b5df756d9ad))
    - Merge pull request #2 from Him7n/main ([`cbb4c3a`](https://github.com/rootCircle/bgit/commit/cbb4c3a8e3d0b4f5007520e9309dacca0f5d4dd5))
    - Fix : Repository Discover ([`dff1c8b`](https://github.com/rootCircle/bgit/commit/dff1c8b6fd88704748c11a8509f0b2f94572e318))
    - Add new error types and update module visibility ([`dd76e51`](https://github.com/rootCircle/bgit/commit/dd76e5154ec3681aba00f23d945802ee5197305c))
    - Fix windows ci ([`add40ef`](https://github.com/rootCircle/bgit/commit/add40ef35161ba0efa46f5b363dd6c560f0279fb))
    - Windows runners as well ([`c555361`](https://github.com/rootCircle/bgit/commit/c555361aa97d43c1b86840401c7ebe883c544dfe))
    - Add error types ([`1c5ac77`](https://github.com/rootCircle/bgit/commit/1c5ac77681ee1f446cd5af527e151d456cf69838))
    - Add new error types and update module visibility ([`b93abfc`](https://github.com/rootCircle/bgit/commit/b93abfc35d659087988f1a1bbad73aa265dc4cda))
    - Fix windows ci ([`1b7e460`](https://github.com/rootCircle/bgit/commit/1b7e460a2394bea0c9e69495513a2f3e326067fe))
    - Windows runners as well ([`a47e112`](https://github.com/rootCircle/bgit/commit/a47e1126912a9e84a7dd48e8ad38386ed6c5057e))
    - Add error types ([`b7a64a7`](https://github.com/rootCircle/bgit/commit/b7a64a7b7a2973a4923b7a3abad6656c60656c76))
    - Fix : linting error ([`ff0c256`](https://github.com/rootCircle/bgit/commit/ff0c256a17b6e4bb89e8338e540e33086e808705))
    - Implement full git add logic using libgit2 ([`daa823f`](https://github.com/rootCircle/bgit/commit/daa823ffdb57d1f71c5c889eb9e7a53a3a25dbbb))
    - Add shields badge ([`2cb3569`](https://github.com/rootCircle/bgit/commit/2cb356903243c6a8aee7e7d67f78930fc43e41e5))
    - Fix tags ([`701e5df`](https://github.com/rootCircle/bgit/commit/701e5dfe2b1aeb70a58c45d1dad705b7a2a377d7))
    - Code cov check ([`f8bdce2`](https://github.com/rootCircle/bgit/commit/f8bdce230d3edc08a2944c335e648e2835eabd48))
    - Minor changes ([`3d5ffc4`](https://github.com/rootCircle/bgit/commit/3d5ffc4c732f186ff1c17ae3d9e16c3d0fb17b47))
    - Fix ([`ea17e15`](https://github.com/rootCircle/bgit/commit/ea17e150e9839dede10ed3f568d67f22bc8b7416))
    - Migrate to rust 2024 and some ci ([`a8528e3`](https://github.com/rootCircle/bgit/commit/a8528e3deb827643cd4fda69245fb86218531961))
    - Implemented hook execution for Windows ([`0693a3d`](https://github.com/rootCircle/bgit/commit/0693a3d230fac16f4a6f9334b31c6f618cd74ffe))
    - Fix typo ([`8cff501`](https://github.com/rootCircle/bgit/commit/8cff501b2278c3407ebaeb582f2e0abda5f8e27d))
    - Add arch docs ([`e79bafc`](https://github.com/rootCircle/bgit/commit/e79bafc4122361dbfda32b87c40f415397a94413))
    - Fix prompt dialog mangled into progress bar ([`ec365a0`](https://github.com/rootCircle/bgit/commit/ec365a0a575fc19ace3ce458041f3c916260692e))
    - Metadata changes ([`376da84`](https://github.com/rootCircle/bgit/commit/376da8460a8e0bd4c45d9cbc582365a959e399a3))
    - Fix os parity checks ([`fb5ee8d`](https://github.com/rootCircle/bgit/commit/fb5ee8d35b9c955d70973cd3ce330767ee1d10de))
    - Improved ui for cli ([`dd2a5dc`](https://github.com/rootCircle/bgit/commit/dd2a5dcf2a2938686ca938184e4df79812161fe3))
    - Renamed commit hook name format ([`e95d8c3`](https://github.com/rootCircle/bgit/commit/e95d8c35b1c924de1feaf011872781f711f2a6ad))
    - Common action store implemented ([`82c701a`](https://github.com/rootCircle/bgit/commit/82c701afb5230e3e759875103806c9f680657aec))
    - Heap for error type and refactor and hook executor ([`3b8196f`](https://github.com/rootCircle/bgit/commit/3b8196f3d7008f404af5baa943bd867c8e25098b))
    - Fixed error design ([`966f828`](https://github.com/rootCircle/bgit/commit/966f828271c587a81469a6042c5638b3e915f655))
    - Add more graphs ([`37de9a2`](https://github.com/rootCircle/bgit/commit/37de9a29ad57d34b4fb6412bab4444fae9a4a90e))
    - Git stash ([`fe03cc3`](https://github.com/rootCircle/bgit/commit/fe03cc3cae05e3ff9126431d1dfdf2bc5222265c))
    - Some def changes ([`5721457`](https://github.com/rootCircle/bgit/commit/5721457c63a7312fb31781c1f489a0d3e626a02c))
    - Some def changes ([`ecdae38`](https://github.com/rootCircle/bgit/commit/ecdae38edd9316d7026cb8bfa75662a2f0465ee6))
    - Is git repo ([`420add0`](https://github.com/rootCircle/bgit/commit/420add022d43ecb2b242876736e7f7fa58809824))
    - Add dummy task ([`577d1ee`](https://github.com/rootCircle/bgit/commit/577d1ee9f0443ffab76983d55dd589d726c0cc10))
    - Some fix in data structures ([`d2badef`](https://github.com/rootCircle/bgit/commit/d2badef3dc6bb7c150199dcf63e8dea7f44c0b31))
    - Intial prototype ([`fea9541`](https://github.com/rootCircle/bgit/commit/fea9541659a4f7117dc7ce596c15c5afe76273e4))
    - Welp ([`ae50a32`](https://github.com/rootCircle/bgit/commit/ae50a3267df8e7f48878a1101b2d623da68e05ac))
    - Code structure ([`ff7a9b5`](https://github.com/rootCircle/bgit/commit/ff7a9b546891852208c01596a0d10ba387a6eddc))
    - Add sample cmd usages and deps ([`4bb574f`](https://github.com/rootCircle/bgit/commit/4bb574f5a0a2cd318ff7b18286c6856cd56c4aa0))
    - Add rules ([`3685f3c`](https://github.com/rootCircle/bgit/commit/3685f3c011840588ff892c7c57259dd62c6f2477))
    - Add workflows ([`fc72b4b`](https://github.com/rootCircle/bgit/commit/fc72b4b6acf1914f847cfbd63653b314240eb338))
    - Add action scripts for test and build ([`fb00861`](https://github.com/rootCircle/bgit/commit/fb00861d9ca0b9498f2efb87543735a6ce3849c3))
    - Add MIT license ([`ec2b431`](https://github.com/rootCircle/bgit/commit/ec2b4318007176ab9fcc8673c37e15e07ad90c14))
    - Initial commit ([`22aa5c0`](https://github.com/rootCircle/bgit/commit/22aa5c046d2d68e834a88136526bee58658637df))
</details>

## v0.2.0 (2025-03-07)

<csr-id-1c5ac77681ee1f446cd5af527e151d456cf69838/>
<csr-id-b7a64a7b7a2973a4923b7a3abad6656c60656c76/>
<csr-id-701e5dfe2b1aeb70a58c45d1dad705b7a2a377d7/>
<csr-id-a8528e3deb827643cd4fda69245fb86218531961/>
<csr-id-8cff501b2278c3407ebaeb582f2e0abda5f8e27d/>
<csr-id-fb5ee8d35b9c955d70973cd3ce330767ee1d10de/>
<csr-id-e95d8c35b1c924de1feaf011872781f711f2a6ad/>
<csr-id-4bb574f5a0a2cd318ff7b18286c6856cd56c4aa0/>
<csr-id-3685f3c011840588ff892c7c57259dd62c6f2477/>
<csr-id-c555361aa97d43c1b86840401c7ebe883c544dfe/>
<csr-id-a47e1126912a9e84a7dd48e8ad38386ed6c5057e/>
<csr-id-f8bdce230d3edc08a2944c335e648e2835eabd48/>
<csr-id-ea17e150e9839dede10ed3f568d67f22bc8b7416/>
<csr-id-376da8460a8e0bd4c45d9cbc582365a959e399a3/>
<csr-id-fb00861d9ca0b9498f2efb87543735a6ce3849c3/>
<csr-id-8045ea32b7a5e4b259934541b72ad9d285d84bda/>
<csr-id-dd4c718df7b0f27c9498cfa481866c47dbda18bd/>

### Chore

 - <csr-id-1c5ac77681ee1f446cd5af527e151d456cf69838/> add error types
 - <csr-id-b7a64a7b7a2973a4923b7a3abad6656c60656c76/> add error types
 - <csr-id-701e5dfe2b1aeb70a58c45d1dad705b7a2a377d7/> fix tags
 - <csr-id-a8528e3deb827643cd4fda69245fb86218531961/> migrate to rust 2024 and some ci
 - <csr-id-8cff501b2278c3407ebaeb582f2e0abda5f8e27d/> fix typo
 - <csr-id-fb5ee8d35b9c955d70973cd3ce330767ee1d10de/> fix os parity checks
 - <csr-id-e95d8c35b1c924de1feaf011872781f711f2a6ad/> renamed commit hook name format
 - <csr-id-4bb574f5a0a2cd318ff7b18286c6856cd56c4aa0/> add sample cmd usages and deps
 - <csr-id-3685f3c011840588ff892c7c57259dd62c6f2477/> add rules

### Documentation

 - <csr-id-b6a4b5561581c9d96e4ff3794c083d7cccc4356e/> windows hook_executor is implemented and fairly stable now
 - <csr-id-2cb356903243c6a8aee7e7d67f78930fc43e41e5/> add shields badge
 - <csr-id-e79bafc4122361dbfda32b87c40f415397a94413/> add arch docs
 - <csr-id-fc72b4b6acf1914f847cfbd63653b314240eb338/> add workflows

### New Features

 - <csr-id-f39eacb0f6235a8be0ea4eafea7cad363923f13a/> add update_cwd_path method to GitClone and InitGitRepo for setting current working directory
 - <csr-id-dd76e5154ec3681aba00f23d945802ee5197305c/> add new error types and update module visibility
 - <csr-id-b93abfc35d659087988f1a1bbad73aa265dc4cda/> add new error types and update module visibility
 - <csr-id-daa823ffdb57d1f71c5c889eb9e7a53a3a25dbbb/> implement full git add logic using libgit2
   This commit introduces a complete implementation of git add functionality in the GitAdd event. Instead of using a placeholder command, the new code leverages libgit2 (via the git2 crate) to open the repository, retrieve the index, add all files recursively, and write the index to disk. Detailed error handling is provided via BGitError.
 - <csr-id-dd2a5dcf2a2938686ca938184e4df79812161fe3/> improved ui for cli
 - <csr-id-82c701afb5230e3e759875103806c9f680657aec/> common action store implemented
 - <csr-id-3b8196f3d7008f404af5baa943bd867c8e25098b/> Heap for error type and refactor and hook executor
 - <csr-id-ff7a9b546891852208c01596a0d10ba387a6eddc/> code structure
 - <csr-id-ec2b4318007176ab9fcc8673c37e15e07ad90c14/> add MIT license
 - <csr-id-22aa5c046d2d68e834a88136526bee58658637df/> initial commit

### Bug Fixes

 - <csr-id-add40ef35161ba0efa46f5b363dd6c560f0279fb/> fix windows ci
 - <csr-id-1b7e460a2394bea0c9e69495513a2f3e326067fe/> fix windows ci
 - <csr-id-ec365a0a575fc19ace3ce458041f3c916260692e/> fix prompt dialog mangled into progress bar

### Other

 - <csr-id-c555361aa97d43c1b86840401c7ebe883c544dfe/> windows runners as well
 - <csr-id-a47e1126912a9e84a7dd48e8ad38386ed6c5057e/> windows runners as well
 - <csr-id-f8bdce230d3edc08a2944c335e648e2835eabd48/> code cov check
 - <csr-id-ea17e150e9839dede10ed3f568d67f22bc8b7416/> fix
 - <csr-id-376da8460a8e0bd4c45d9cbc582365a959e399a3/> metadata changes
 - <csr-id-fb00861d9ca0b9498f2efb87543735a6ce3849c3/> add action scripts for test and build

### Refactor

 - <csr-id-8045ea32b7a5e4b259934541b72ad9d285d84bda/> remove old InitGitRepo action and replace with new prompt implementation
 - <csr-id-dd4c718df7b0f27c9498cfa481866c47dbda18bd/> remove name, description from trait definition for new

