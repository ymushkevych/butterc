# Eprint

## About

`eprint` is the butter intrinsic for printing string literals to the terminal via stderr (fd=2)

`eprint` does not accept expressions of any kind, including string concatenation, and only accepts singular string literals

`eprint` automatically appends a newline character ('\\n')

## Usage

`eprint` is used as `eprint STR_LIT;` where `STR_LIT` is a string literal

`eprint` is only usable within function scopes

### Example 
    int fnc main() {
      eprint "There was an error"; % <- this
      ret 0;
    }
    
    % Terminal Output (through stderr):
    % There was an error 
    %
