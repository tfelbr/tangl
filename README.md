# tangl

[//]: # (<p align="center">)

[//]: # (  <img src="" alt="" />)

[//]: # (</p>)

Welcome to **tangl**, a tool to manage git repositories as feature models!

## How To Use

### The Basics

Git is a ***version control system*** (VCS) and is therefore great in managing consequitive versions of your project.
This is also known as ***variability in time***.

You also may have multiple ***variants*** of the same artifact next to each other.
For example, a specific module adapted for different OSs/hardware. 
This is called ***variability in space*** and is usually organized as so-called ***feature models***.

Git can model parallel variants with its branches, but traditionally, they are meant to be merged into main, sooner or later.
This forcibly entangles artifacts that you might want to have separated by default, and only assemble as needed.

Git does not remember the parent-child relationships between branches, so using git-native commands to manage a feature model is very combersome.
This is where **tangl** comes into play: it introduces a naming convention for branches, and parses them internally to recognize those relationships!

### The Repository Tree

**tangl** introduces a number of types each node in this tree may possess.

```
main [area]
├── feature [feature root]
│   ├── foo [feature]
│   └── bar [abstract feature (no branch)]
│       └── baz [feature]
└── product [product root]
    └── myproduct [product]
```

- **Area**: the top most branch, used to organize the remaining structure underneath it.
    The first one is usually ``main`` or ``master``, but additional ones like ``develop`` are possible.
- **Feature Root**: All ***feature*** branches are organized under this node.
- **Product Root**: All ***product*** branches are organized under this node.
- **Feature**: 
- **Product**

### Mechanics

#### Navigating the Tree

As opposed to native git, where each branch stands alone, navigating the tree works like the ``cd`` command on the shell.
You can checkout children of the current branch by using the child's name alone. You do not need to type the entire path.

All ***areas*** have the common root ``/``.
You can prefix paths with ``/`` to make them absolute.
The relative identifiers ``.`` and ``..`` are also possible.
For example:

```bash
# When on main:
# These navigate to /main/feature/foo
tangl checkout feature/foo
tangl checkout /main/feature/foo

# When on /main/feature/foo:
# These navigate to /main
tangl checkout ../..
tangl checkout /main
```
This convention is not limited to checkout.
It applied throughout all commands.

#### Consistency Preservation

**tangl** attempts to preserve consistency between features and products using merge-tests and commit/patch comparison.
Merge-tests can be executed manually via the ``test`` command, which test-merges branches to discover conflicts.
These checks are executed automatically as part of some other operations.

#### Developing Features

Feature development works similar to git.
You checkout a feature branch, make modifications, and commit to it.
You can add and remove features via the ``feature`` command.

If you commit on a feature, this feature is test-merged against all others and you are notified if you introduced conflicts.

Typical workflows involving branches like ``develop`` or ``release`` are possible in theory.
Currently, there is no explicit support, but we plan to use ***areas*** for that.

#### Deriving Products

To assemble a product out of features, you first create a product via the ``product`` command.
After checkout, you run the ``derive`` command and pass the features you like.
If you use bash, the command suggests features via auto-completion for you.

Product derivation is a staged process.
**tangl** performs test-merges of all participating features at the start, so you get a preview of how many conflicts you can expect.
When passing the ``-o/--optimize`` flag, it will attempt to optimize the merge order to move conflicts at the end of the chain.
This reduces follow-up conflicts and can recognize if fixes are already present on the product or a feature.

#### Updating Products

#### *Untying*: Moving Change from Product to Feature

## Getting Started

### Running the Example in Docker
We provide a toy example inside a docker container, so you don't need to install anything on your system to try it out.

**Requirements**
- docker compose
- docker buildx
- make

In the repository's root, run
```bash
make example
```
This builds and spins up a docker container, containing the ``tangl`` binary and command, bash completion, and the example repository.

The container will expose the example repository under ``target/example`` in this repository's root.
You can use any editor/IDE to make modifications and use the container to run ``tangl`` commands.

### Local Build and Installation
**Requirements:**
- Latest version of Rust and Cargo 1.x
- bash for dynamic completion
- ``~/.cargo/bin`` on your PATH
- make

In the repository's root, run
```bash
make
```
This builds the binary and installs it under ``~/.cargo/bin``.
This also copies a script for bash completion into ``~/.local/share/bash-completion/completions``.

Verify your installation by running ``tangl``.
This should print a help.

### For Developers:
