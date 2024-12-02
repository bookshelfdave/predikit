## Notes

> These aren't really meant for other folks, lots of this text is misc ideas / etc that
I wanted to record somewhere.


## The problem

- Complex tooling! Salt, Puppet, etc, many moving parts, lots to read and learn to Get It Right.
- IMO, Terraform, Ansible etc try and implement All The APIs
- "batteries are included" - too much code to maintain and get out of date
- Shell scripts are powerful, but messy.
  - Everyone's shell scripts (and Perl) look different.
  - IMO they are not the best way to convey a series of checks.


## Design principles + misc notes

- Don't build plugins for different products, your CLI already has a bajillion amazing things that it can do.
- Ship a *tiny* core of functionality. There are a ton of tools out there already that do most of the
- Usage of the tool should always favor simplicity
  - although there may be complicated things happening under the hood
things we need day to day, build upon those instead.
- Prediki is the coordinator for checks / tests
  - Checks / tests are "front and center"
  - what is being tested is easy to find, the implementation of the test might be in a shell script
    or another programming language altogether.
- Tests are front and center
- Extend predikit with Tool definitions
- build in retries and timeouts
- pre/post/pass/fail/error hooks
- concurrency is out of scope (for now?)
- If your command takes more than 1 line to exec, put it into a script.
- super simple type system
- compile to catch errors before running tests
- it's a simple way to interact with CLI tools, oriented as a testing framework

## Out of scope

- dealing with SSH
- dealing with sudo
- Concurrent tests (maybe use Gnu Parallel?)
- more than 1 script line per field
- supporting specific products


is a WIP, and is not usable at all.

# TODOs
- Required:
  - [x] main return code!
  - [x] position in lexer errors
  - [x] compile only
  - [ ] output test match thing
        let matched = Regex::new(r"^regexStr$").unwrap().is_match("string");
  - [ ] Checks
    - FS
      - is_file
      - is_directory
      - is_symlink
      - is_readable
      - is_writable
      - is_executable
      - permissions
  - [ ] InvalidInteger needs to store a ContentAddress
  - [ ] no color
  - [ ] predikit --docs exists?
  - [ ] fix triggers
  - [ ] cleanup event stuff
    - [ ] all checks are sent to the formatter after compile, this is inefficient for large check files
  - [ ] Don't allow reuse of builtin/materialized property names
  - [ ] Default values
  - [ ] Tool definitions
    - [ ] fail_msg_template
    - [ ] pass_msg_tempalte
    - [ ] err_msg_template
    - [ ] diffs! ^^^
    - [ ] pass env into templates?
    - [ ] title?
    - [ ] custom shell
  - [ ] shell timeouts could use: use https://docs.rs/wait-timeout/0.2.0/wait_timeout/

- [ ] tests everywhere
  - [x] typechecking
  - [ ] compiler: tools and tool param types
  - [ ] tools in general
  - [ ] conversions
  - [ ] hooks
  - [ ] retry

- [ ] read from stdin?

- [ ] Hooks:
  - [x] on_pass, on_fail, on_error
  - [x] on_init, on_term
  - [ ] custom shell for hooks
  - [ ] set current dir and env?
  - [ ] pass check output etc to subcommand


- [ ] Compiler
  - [ ] standardize ownership
  - [ ] ensure formal / actual params are not overwritten
  -   [ ] I think it's the same problem, but you shouldn't be able to specify the same parameter more than once
  - [x] Add a duration type
- [ ] error checking
  - [x] multiple errors during compile
  - [x] make the errors not suck (part 1)
  - [ ] make the errors not suck (part 2)


- Nice to haves:

- [ ] Global config (for shell execs etc)?
- [ ] config file?
- [ ] fancy terminal output
  - [ ] cli params to control output
  - [ ] render shell output?
- [ ] json output
- [ ] Add filename to instance + instance output
- [ ] Use Path everywhere instance of a String?
- [ ] run_id, instance_id env vars for shell checks
- [ ] debug, clone, etc on structs
- [ ] struct comments

- Future
  - [ ] Timeouts
    - [ ] are complicated
    - [ ] shell timeouts could use: use https://docs.rs/wait-timeout/0.2.0/wait_timeout/


# misc

https://crates.io/crates/ptree
https://crates.io/crates/prettytable
https://crates.io/crates/parse_duration

# License

Apache License 2.0
see [LICENSE](LICENSE)
