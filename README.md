# Voran
Voran is an installer project designed to work seamlessly with JellyFish packages

## Features
 - [x] Install and Uninstall
 - [x] Support for JellyFish packages
 - [x] Add binary files to path without reload of shell required
 - [x] Support for Wharf Packages
 - [x] Support for Executable installers
 - [x] Multiple repositories
 - [x] Install specific version of application

**Warning:** Voran is still in a demo stage, none of it is expected to be production ready, and many things are planned to change.

## Getting Started

### Installing
If you have Cargo installed
```
git clone https://github.com/DukeofStars/voran.git
cd Voran
cargo install --path .
```
Otherwise download the latest binaries from https://github.com/DukeofStars/voran/releases/

### Managing Packages
To install

`voran install <package>`

To uninstall

`voran uninstall <package>`

To list packages

`voran list [--local|--remote]`

### Managing remotes
Remotes are Git Repositories.

To add a remote

`voran remote add <alias> <git_repo_url>`

To remove a remote

`voran remote remove <alias>`

To list remotes

`voran remote list`

## Contributing
Feel free to contribute, at the moment this project is more of a hobby for me, so it would be much appreciated.