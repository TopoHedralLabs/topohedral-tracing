# Introduction

This crate provides a tracing mechanism for the topohedral collection of crates.
It is similar to the `env_logger` crate, and uses a similar syntax to specify the logging 
filters. However it differs from `env_logger` in that it also provides a compile-time option to 
enable or disable logging. It uses the `log` crate interface, therefore it provides five 
macros for logging which are listed in decreasing order of verbosity:

- `trace!` 
- `debug!`
- `info!`
- `warn!`
- `error!`. 

# Usage

## Compile time configuration

The printing code is only compiled if the `enable_trace` feature is enabled. Otherwise the 
code resolves to nothing. Any crate which uses this crate can enable logging by having the 
following in their `Cargo.toml` file:

``` toml
[dependencies]
topohedral-tracing = {<version etc>}

[features]
enable_trace = ["topohedral-tracing/enable_trace"]
```

and compiling with the `enable_trace` feature. 

## Runtime configuration

Even with logging enabled at compile time, runtime logging filter will be dafault print 
nothing. This can be changed by setting the `TOPO_LOG` environment variable. This variable 
has the following syntax:

```shell
export TOPO_LOG=<target>=<level>,<target>=<level>,...
```
Additionally, there is a special target `all` which can be used to enable all logging of a 
given level. So, for example, to log everything at level `debug` we can do:

```shell
export TOPO_LOG=all=debug
```
