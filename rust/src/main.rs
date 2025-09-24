mod option;
use option::menu;

mod hiiro;
use hiiro::hello_hiiro;

mod utils;
mod locale;
use locale::init_locale;

fn main() {
    // 初始化语言环境
    init_locale();
    
    hello_hiiro();
    menu();
}



