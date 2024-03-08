#include <string>
#include <vector>
#include <iostream>
#include <regex>
#include <fstream>
#include <functional>

#include "inquirer.h"



class Option
{
public:
    Option();
    // ~Option();
    void base_option();
    void second_option();
private:
    std::string homeDir;
    std::vector<std::string> vailHosts;
    alx::Inquirer inquirer_base = alx::Inquirer("menu");
    alx::Inquirer inquirer_sec = alx::Inquirer("");
    std::string base_choice;
    std::string second_choice;

    std::string get_home_dir();
    std::vector<std::string> get_vail_Hosts();
};