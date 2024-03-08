mod option;
use option::menu;

mod hiiro;
use hiiro::hello_hiiro;

mod utils;

fn main() {
    hello_hiiro();
    menu();

}



