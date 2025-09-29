# Work Group Distributor

This repository contains a Rust application that reads two lists of names from text files, randomly distributes work assignments, and prints the results in a formatted output.

## Setup

### 1. Install Rust

If you haven't installed Rust yet, you can install it by following the instructions at [rust-lang.org](https://www.rust-lang.org/learn/get-started).

### 2. Create the Input Files

Create two text files, `file_a.txt` and `file_b.txt`, in the root directory of the repository. Each file should contain a list of names, one per line.

Example of `file_a.txt`:

```txt
Alice
Bob
Charlie
```

Example of `file_b.txt`:

```txt
David
Eve
Frank
```

### 3. Running the Program

To run the program:

1. Clone the repository:

   ```bash
   git clone https://github.com/onelrian/VividShift.git
   cd VividShift
   ```

2. Build and run the application using Cargo:

   ```bash
   cargo run
   ```

The program will read the names from `file_a.txt` and `file_b.txt`, distribute the work assignments, and print the result in the following format:

### Example Output

```
**📊 Work Distribution Results**

**Toilet A**: Alice, Bob
**Toilet B**: David, Eve
**Parlor**: Charlie, Frank
**Frontyard**: Alice, Eve, Frank, Bob
**Backyard**: Charlie
**Tank**: Alice, David
**Toilet B**: Eve, Frank, David
**Bin**: Charlie
```

## How to Customize

### Modifying Work Assignments

You can change the number of people assigned to each task by modifying the `work_assignments` array in `group.rs`. The format for each entry is:

```rust
("Task Name", number_of_people)
```

For example, to change the number of people assigned to `Parlor`, modify the entry like this:

```rust
("Parlor", 3),  // Assign 3 people instead of 4
```

### Changing the Input Files

If you want to use different files or modify the structure of your input files, make sure to update the `read_names_from_file` function in `files.rs` to handle the changes accordingly.

### Modifying Output Format

The output format can be changed in the `print_assignments` function in `output.rs`. This function prints the assignments in a simple format, but you can modify it to match any structure you prefer.

