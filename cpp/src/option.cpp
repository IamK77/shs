#include "option.h"
// #include "emoji.h"
// #include "core.h"

Option::Option()
{
    this->homeDir = this->get_home_dir();
    this->vailHosts = this->get_vail_Hosts();
}

std::string Option::get_home_dir() {
    const char* homeDir = getenv("USERPROFILE");
    if (homeDir == NULL)
    {
        std::cerr << "USERPROFILE not found, trying HOMEPATH" << std::endl;
    } else {
        // std::cout << homeDir << std::endl;
    }
    std::string path = std::string(homeDir) + "\\.ssh\\config";

    return path;
}

std::vector<std::string> Option::get_vail_Hosts() {
    std::string path = get_home_dir();

    std::ifstream file(path);
    std::string line;

    std::regex pattern(R"(Host\s*([^#]*))");
    std::vector<std::string> vailHosts;

    while (std::getline(file, line))
    {   
        size_t found = line.find("#");
        if (found != std::string::npos)
        {
            line = line.substr(0, found);
        }
        std::smatch results;
        if (std::regex_match(line, results, pattern))
        {
            vailHosts.push_back(results[1]);
        }
    }

    return vailHosts;
}

void Option::base_option() {
    this->inquirer_base.add_question({"base", "What do you want to do?",
                {"Connect to a server", "Add a new server", "Exit"}});
    this->inquirer_base.ask();
    this->base_choice = this->inquirer_base.answer("base");
}

void Option::second_option() {
    if (this->base_choice == "Connect to a server")
    {
        this->inquirer_sec.add_question({"second", "Which server do you want to connect?",
                    this->vailHosts});
        this->inquirer_sec.ask();
        this->second_choice = this->inquirer_sec.answer("second");
        std::string command = "ssh " + this->second_choice;
        int result = std::system(command.c_str());

        if (result != 0) {
            std::cout << "oops, something went wrong!" << std::endl;
        }

    } else if (this-> base_choice == "Add a new server")
    {
        /* code */
        std::cout << "wait a update" << std::endl;

    } else if (this->base_choice == "Exit")
    {
        // std::cout << emojicpp::emojize(":innocent:")  << std::endl;
        // fmt::print("😋");
        // fmt::v10::vprint(fmt::format("{}\n", "Hello, world!"));
        // fmt::print("😋");
        std::exit(0);
    }
    
}