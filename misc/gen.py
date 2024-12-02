def generate_checks():
    # List of example paths to check
    paths = [
        "/home/dparfitt",
        "/etc/hostname",
        "/var/log",
        "/usr/local/bin",
        "/opt",
        "/tmp",
        "/root",
        "/boot",
        "/dev",
        "/proc"
    ]

    # Open file for writing
    with open("checks.txt", "w") as f:
        # Write the opening bracket
        f.write("all {\n")

        for i in range(10000):
            # Generate 10 checks
            for path in paths:
                # Write each check with proper formatting
                f.write("  test exists? {\n")
                f.write(f"    path: \"{path}\"\n")
                f.write("  }\n\n")

        # Write the closing bracket
        f.write("}\n")

if __name__ == "__main__":
    generate_checks()
    print("Checks have been written to 'checks.txt'")
