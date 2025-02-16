# Layered Molecular Editor (LME)

This is a program for automatically constructing molecular models and prepare for first-principle computations by delaretive-programming input file and resources organized by it.

## Installation

### Software requirements

LME is written in Rust and should be easily to compile with Rust toolchains, we will also provide binary files of release versions in the GitHub release. 

Besides the LME itself, you still need to install follow software:

- OpenBabel 3.X: OpenBabel will help this program to generate output files in standard format.
- GNU sed (Optional): GNU sed allows users to custom the output file with regular expression in the input file.

> Please add the installation directory to PATH environment variable

> You may find LME can now work without OpenBabel, but in later version it will become a strong dependency.

### Hardware requirements

- CPU: The LME itself doesn't contain any platform-specified code, but as most first-principle calculation software works only on AMD64 platform, we only test it on AMD64 CPUs. There is no minimum CPU performace requirements and number of cores is more important than frequency for LME itself, but the user-developed plugins may require better single-core performance.
- Memory usage: The runtime memory is mainly used to store the layer index of each model and information about recently built and used structures cached based on the LRU algorithm, the former usually increases with the number of models and modelling steps, while the latter can be controlled in terms of the number of reservations using the `LME_CACHE_SIZE` environment variable. In most tasks, the peak running memory will not exceed 2 GB.
- Hard disk: The layers are stored in a embedded database on the hard disk, which usally takes less than 1GB space. Though the total amount of data is small, the embedded database will wait the file system to synchronise the write operations to disk, so the SSDs can significantly improve the performance.

### Installation

In the release package, serveral binary files are provided. You can put them in anywhere you like, and add the directory to PATH environment variable.

The binary files should contain:

- lmers: The main program of LME.
- obabelme: Tools convert other molecular files between common format to LME format.

The binary files of x64 Linux platform is provided in releases.

### Build from source

The core of LME is written in Rust, please follow the [instruction](https://www.rust-lang.org/tools/install) from rust-lang.org to setup the environment.

When finish the installation of Rust, switch to the directory of the LME and execute `cargo build`, the binary files will be generated under the folder of `target/debug`

To generate a release version for better performance, execute `cargo build --release`, the binary files will be generated under the folder of `target/release`.

## Examples

We provide here 3 examples of building chemical reaction datasets using LME placed in the examples directory.

1. classical DA reaction: `example/DA`
2. Claisen rearrangement reaction: `example/Claisen`
3. RuPNP catalyst catalyzed asymmetric ketone hydrogenation (AKH): `example/Ru`

The optimized XYZ coordinates is provided in `output` directory of these folders.

To reproduce the construction process of input files `lmers -i lme_workflow.inp.yaml` (Using `cargo run --bin lmers -- -i lme_workflow.inp.yaml` if you are working in a Rust programming environment) to complete the construction of the input file. The construction process requires Python and OpenBabel (conda is recommended) to be installed, and the build of AKH also requires XTB 6.7.1, and you need to make sure that these programs are installed on your system and can be called directly before running LME.
