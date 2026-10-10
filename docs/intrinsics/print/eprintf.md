# Eprintf

## About

`eprintf` is the butter intrinsic for printing formatted strings and expressions to the terminal via stderr (fd=2)

`eprintf` accepts the following expressions and items:
  - variables
  - constants
  - raw binary expressions (+, *, -, /)
  - literals
  - functions

`eprintf` automatically appends a newline character ('\\n')

## Usage

`eprintf` is used as `eprintf ASV_LIST;` where `ASV_LIST` is a list of values separated by an ampersand (&)

`eprintf` is only usable within function scopes

### Example 
    int fnc main() {
      str var error->"Something went wrong"
      eprintf "Oops we found an error: "&"error; % <- this
      ret 0;
    }
    
    % Terminal Output (through stderr):
    % Oops we found an error: Something went wrong 
    %
