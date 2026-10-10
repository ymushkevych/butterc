# Out

## About

`out` is the butter intrinsic for exiting programs

`out` accepts a singular input that evaluates to an integer

`out` can in theory parse any unsigned 64-bit value but x64 only allows exiting with 8-bit integers

## Usage

`out` is used as `out CODE;` where `CODE` is an integer

`out` accepts the following items and expression:
  - integer variables
  - integer constants
  - binary expressions
  - integer literals
  - integer functions

`out` is usable within any scope

### Example
    int fnc main() {
        out 0; % <- here
    }

    % Terminal Output:
    % ... running 'echo $?' to get exit code
    % 0
    %
