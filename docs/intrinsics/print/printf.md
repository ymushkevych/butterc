# Printf

## About

`printf` is the butter intrinsic for printing formatted strings and expressions to the terminal via stdout (fd=1)

`printf` accepts the following expressions and items:
  - variables
  - constants
  - raw binary expressions (+, *, -, /)
  - literals
  - functions

`printf` automatically appends a newline character ('\\n')

## Usage

`printf` is used as `printf ASV_LIST;` where `ASV_LIST` is a list of values separated by an ampersand (&)

`printf` is only usable within function scopes

### Example 
    int fnc main() {
      str var name->"tom"
      printf "Hello "&"name; % <- this
      ret 0;
    }
    
    % Terminal Output (through stdout):
    % Hello tom 
    %
