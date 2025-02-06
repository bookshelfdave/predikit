# Predikit

Predikit is a minimalistic systems testing language that prioritizes simplicity and the use of existing CLI tools.
By shipping a tiny core of functionality and focusing on intuitive, test-oriented workflows, Predikit makes it easy
to define, compile, and execute tests using CLI tools you're already familiar with.

Predikit is a work in progress as of Feb 2025, and is not stable for _any_ use.

## Why?

I’ve always wanted a lightweight solution to run checks against a system without the overhead of large runtimes. Tools
like Serverspec and Inspec are great, but they bring unnecessary complexity for many use cases.

During my time leading the Terraform SDK (DevEx) team, where we built Golang libraries to help developers extend
Terraform with their APIs, I realized how much effort goes into integrating each cloud provider or service. While
I wasn't looking to build tooling that interacted with cloud providers specifically, tt made me think — why not
leverage existing CLI tools instead of constantly writing new code? This approach would simplify testing and cut
down on the need for custom plugins, leading to Predikit's creation.

I also wanted to create an infrastructure tool that doesn’t rely on a bunch of other services (DBs, queues, K8s, etc)
or a complex architecture just to run something as simple as a "hello world." My goal was a single compiled binary — quick,
efficient, and straightforward.

## What does it look like?

> I don't have any documentation yet, so this blob of Predikit code will have to do for the time being. I do plan on writing an
[mdbook](https://github.com/rust-lang/mdBook)-style tutorial and guide once the bits have settled.

Here's a contrived set of tests that show off several of Predikit's features:

```
all {
    // all tests in this group must pass. Other groups are `any` and `none`. Groups can be
    // arbitrarily nested.

    title: "Demo the tiny set of core tests available 😀"
    test exists? {
      // p(..) or path(..) allow for expansion of shell variables in a string
       path: p( $HOME/.zshrc )
    }

    // is the file /bin/zsh executable?
    test executable? {
      path: "/bin/zsh"
    }

    // is the `zsh` command on the path?
    test on_path? {
      path: "zsh"
    }

    // Check if a file does NOT exist
    test not exists? {
       path: "/home/dparfitt/.foo"
    }

    // Only a few operations exist out of the box, but checking open
    // ports and a few other misc networking tests could be useful.
    test not port_open? {
        port: 6666
        // on_* properties are hooks that can run arbitrary shell commands
        on_pass: "echo 'Development server port is not in use'"
        on_fail: "echo 'Port 6666 is in use, please stop whatever service is using port 6666'"
    }

    // Prepending an @ to a test enables automatic retrying until success and makes the `retries` and `retry_delay`
    // properties available in your test.
    // This tests if `localhost:53` is open 10 times with a 1 second + 50 millisecond + 10ns delay (arbitrary, I know)
    @test port_addr_open? {
        addr_port: "localhost:53"
        retries: 10
        retry_delay: d(1s 50ms 10ns)  // d(..) represents a duration format
    }
}


// Define reusable tools with a "tool" block that can be used with the "test" keyword.
// Tool definitions can live in separate test files, so libraries of common functionality can be built up
// and included in your tests.
// See the tests below for examples of using this tool
tool pacman_installed? {
    // a tool has a Handlebars template that is rendered at runtime using
    // defined properties. In this case, pkg_name is rendered as part of the command
    // A non-zero exit code indicates failure!
    cmd_template: "pacman -Qe {{pkg_name}}"

    // Define a `pkg_name` property for `pacman_installed?` that only accepts strings
    $pkg_name {
        type: String
        required: true
    }
}

all {
    title: "Check development dependencies using a user-defined tool"

    // Check to see if docker is installed using the tool defined above
    test pacman_installed? {
        pkg_name: "docker"
    }

    // The tool can be used like a regular test (including hooks and retries)
    test pacman_installed? {
        pkg_name: "neovim"
        on_fail: "echo try installing the package: sudo pacman -Syu neovim"
    }

    // ensure that vim is not installed (nothing against vim!)
    test not pacman_installed? {
        pkg_name: "vim"
    }

    // If the emacs package isn't installed, try to install it and repeat the test
    @test pacman_installed? {
        pkg_name: "emacs"
        /* if emacs isn't installed, then install it, although passing the --noconfirm to
           pacman is discouraged */
        on_fail: "echo 'emacs not installed, attempting to install'; sudo pacman --noconfirm -S emacs"

        retries: 2
        retry_delay: d(0ms)  // don't sleep at all
    }
}

```

Running the script above with `predikit` looks like this:

```bash
❯ predikit ./checks/first.pk

* Running tests from ./checks/first.pk:
|   [Demo the tiny set of core tests available 😀] [all]
|  -> [exists?] path: $HOME/.zshrc Pass [11μs]
|  -> [executable?] path: /bin/zsh Pass [2μs]
|  -> [on_path?] path: zsh Pass [38μs]
|  ->  not [exists?] path: /home/dparfitt/.foo [1μs]
|  ->  not [port_open?] port: 6666 [49μs]
[on_pass]: Development server port is not in use

|  -> [port_addr_open?] addr_port: localhost:53 Pass [382μs]
|   Pass [1387μs]
|   [Check development dependencies using a user-defined tool] [all]
|  -> [pacman_installed?] pkg_name: docker Pass [23911μs]
|  -> [pacman_installed?] pkg_name: neovim Pass [11949μs]
|  ->  not [pacman_installed?] pkg_name: vim [26580μs]
|  -> [pacman_installed?] pkg_name: emacs Pass [14824μs]
|   Pass [77320μs]
* Finished running tests from ./checks/first.pk
All root checks passed
```

> This is my first take at test output. I have a rough design for output formatters that can be switched via the CLI.

## Building

You'll need a modern verison of Rust + Cargo [installed](https://rustup.rs/):

```
git clone https://github.com/bookshelfdave/predikit
cd predikit
cargo build

 #  Copy ./target/debug/predikit somewhere on to your PATH
```

## The zen of Predikit

- Ship a tiny core of testing functionality
  - enough to get you going / "bootstrap" a system (for some definition of bootstrap)
  - better tooling for everything else already exists in your favorite tools (Python, Ruby, bash/zsh, Terraform, etc)
- Simplicity-first UX
  - within reason
- Predikit is a coordinator for tests: all of your tests are "front and center"
  - what is being tested is easy to find, the implementation of the test might be in a shell script or some other tool
- Compilation should be useful to the end user
  - possibly save some headaches
- Predikit does not maintain state between tests or between invocations.
- Use a few simple types with optional input validation to help reduce errors at compile time
  - String, Int, Bool, Path, Duration, etc
  - duration(), path(), file_perms(), ... functions can validate and convert a specialized input string into a regular String

## Contributions

I'm not accepting any contributions at this time. When Predikit seems like it's in a good state, I'll open
up issues and PRs.

## Significant work before an initial release

- Queries - run commands and test output against string / regex / int predicate

```
test package_version? {
  pkg_name: "some_tool"
} = "1.0.3"

test package_version? {
  pkg_name: "some_other_tool"
} =~ r#^1\.0.*#
```

- all kinds of testing
- command timeouts
- `tool` improvements
  - pass/fail/error hook templates using input params
  - custom shell
- hook and shell cleanup
  - specify shell, shell params, etc
- documentation
- file:line numbers to non-error output
- JSON output, minimal output, no color output

## Out of scope

- plugins
- dealing with SSH
- dealing with sudo
- concurrent tests
- supporting specific products
- probably lots of other things
- Any type of Windows support

# License

This project is licensed under the Apache License 2.0

see [LICENSE](LICENSE)

# Copyright

Copyright (c) 2025 Dave Parfitt
