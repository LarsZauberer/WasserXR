# WasserXR

<p align="center">
    <img src="./logo.svg" width="50%">
</p>

A **dynamic XR Engine** to help you stay in the **flow**

---

WasserXR is a game engine that works with an ECS that handles quick **hot
reloading** and easy to **iterate code**. It is specialized for VR/MR/AR
applications.

This repository is just the ECS which is the core component of the engine.

For the **core library** see the [Core Library Repository]()

## Usage

> By copying, modifying, compiling, distributing, or using the Engine, the User
> agrees to the [License](./LICENSE.md)

- Indie Users are granted a free, non-exclusive, non-transferable license to
  use, modify, compile, and distribute the Engine as part of Games.
- Users exceeding the Indie User threshold must obtain a commercial license from
  the Licensor.

## Installation

[Installation Guide](https://wasserxr.com/getting_started/installation)

### General

Clone both repositories and build them with CMake. Install WasserXR first, since
the Core library depends on it.

```bash
git clone https://github.com/LarsZauberer/WasserXR
git clone https://github.com/LarsZauberer/WasserXR-Core
```

Then build and install WasserXR:

```bash
cd ../WasserXR
cmake -S . -B build
cmake --build build
sudo cmake --install build
```

Build and install WasserXR-Core:

```bash
cd WasserXR-Core
cmake -S . -B build
cmake --build build
sudo cmake --install build
```

### NixOS/Flakes

Add both repositories as flake inputs and expose their packages as `buildInputs`
in your `devShell`.

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    wasserxr.url = "github:LarsZauberer/WasserXR";
    wasserxr-core.url = "github:LarsZauberer/WasserXR-Core";
  };

  outputs = { self, nixpkgs, wasserxr, wasserxr-core }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          wasserxr.packages.${system}.default
          wasserxr-core.packages.${system}.default
        ];
      };
    };
}
```

Run `nix develop` to enter the shell. Headers and libraries are now on your path
and ready to link against.

## Documentation

- [Tutorials](https://wasserxr.com/getting_started/setup)
- [Demo](https://github.com/LarsZauberer/WasserXR-Demo)
- [API-Documentation](https://docs.rs/wasserxr)
