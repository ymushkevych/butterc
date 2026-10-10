# Ret

## About

`ret` is the butter intrinsic for returning values from functions

`ret` "assigns values" to functions so that they can be called in other statements

`ret` accepts the following expressions and items:
  - variables
  - literals
  - functions
  - binary expressions

`ret` does not accept constants, as those are shared globally among all scopes, and therefore have no reason to be returned
additionally, `ret` can only accept items that are of the same type as the function it is in

unless a function has the `null` type, it must contain a `ret`


## Usage

`ret` is used as `ret VALUE;` where `VALUE` is any non-constant data-type from the list above that matches the type of the function being exited from

`ret` is only usable within function scopes

### Example
    str fnc return_example() {
        ret "ok"; % <- here
    }

    int fnc main() {
        printf return_example(); % <- function is called here
        ret 0;
    }
