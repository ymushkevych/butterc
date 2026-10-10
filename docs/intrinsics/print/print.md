# Print

## About

`print` is the butter intrinsic for printing string literals to the terminal stdout (fd=1)

`print` does not accept expressions of any kind, including string concatenation, and only accepts singular string literals

`print` automatically appends a newline character ('\\n')

## Usage

`print` is used as `print STR_LIT;` where `STR_LIT` is a string literal

`print` is only usable within function scopes

### Example 
    int fnc main() {
      print "Hello World"; % <- this
      ret 0;
    }
    
    % Terminal Output (through stdout):
    % Hello World 
    %

