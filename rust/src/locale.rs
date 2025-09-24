use std::env;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Translations {
    pub menu_title: String,
    pub menu_options: Vec<String>,
    pub choose_host: String,
    pub no_hosts: String,
    pub add_host: String,
    pub add_host_prompt: String,
    pub user_prompt: String,
    pub port_prompt: String,
    pub hostname_prompt: String,
    pub host_added: String,
    pub connect: String,
    pub execute_precommand: String,
    pub add_precommand: String,
    pub edit_config: String,
    pub edit_precommand: String,
    pub generate_rsa: String,
    pub exit: String,
    pub invalid_choice: String,
    pub error: String,
    pub success: String,
    pub enter_command: String,
    pub command_added: String,
    pub enter_email: String,
    pub rsa_generated: String,
    pub rsa_failed: String,
    pub no_precommand: String,
    pub choose_command: String,
    pub execute_command: String,
    pub no_suitable_editor: String,
    pub install_editor: String,
    pub invalid_host: String,
    pub invalid_port: String,
    pub empty_fields: String,
}

#[derive(Debug, Clone)]
pub struct Locale {
    pub language: String,
    pub translations: Translations,
}

impl Locale {
    pub fn new() -> Self {
        let lang = Self::detect_language();
        let translations = Self::load_translations(&lang);
        
        Self {
            language: lang,
            translations,
        }
    }
    
    fn detect_language() -> String {
        // 检测系统语言环境
        if cfg!(target_os = "windows") {
            // Windows 系统
            if let Ok(lang) = env::var("LANG") {
                if lang.contains("zh") {
                    return "zh".to_string();
                }
            }
            
            if let Ok(lang) = env::var("LC_ALL") {
                if lang.contains("zh") {
                    return "zh".to_string();
                }
            }
            
            if let Ok(lang) = env::var("LC_CTYPE") {
                if lang.contains("zh") {
                    return "zh".to_string();
                }
            }
        } else {
            // Unix/Linux/macOS 系统
            if let Ok(lang) = env::var("LANG") {
                if lang.contains("zh") {
                    return "zh".to_string();
                }
            }
            
            if let Ok(lang) = env::var("LC_ALL") {
                if lang.contains("zh") {
                    return "zh".to_string();
                }
            }
            
            if let Ok(lang) = env::var("LC_MESSAGES") {
                if lang.contains("zh") {
                    return "zh".to_string();
                }
            }
        }
        
        // 默认返回英文
        "en".to_string()
    }
    
    fn load_translations(lang: &str) -> Translations {
        match lang {
            "zh" => Self::chinese_translations(),
            _ => Self::english_translations(),
        }
    }
    
    fn english_translations() -> Translations {
        Translations {
            menu_title: "Menu".to_string(),
            menu_options: vec![
                "Connect".to_string(),
                "Execute precommand".to_string(),
                "Add a new host".to_string(),
                "Add a new precommand".to_string(),
                "Edit config".to_string(),
                "Edit precommand".to_string(),
                "Generate RSA key".to_string(),
                "Exit".to_string(),
            ],
            choose_host: "Choose a host".to_string(),
            no_hosts: "You don't have any hosts to connect to".to_string(),
            add_host: "Add a new host".to_string(),
            add_host_prompt: "Enter a domain name or IP address for SSH access:".to_string(),
            user_prompt: "Enter the username for SSH access:".to_string(),
            port_prompt: "Enter the port for SSH access:".to_string(),
            hostname_prompt: "Enter the hostname for SSH access:".to_string(),
            host_added: "Host added successfully".to_string(),
            connect: "Connect".to_string(),
            execute_precommand: "Execute precommand".to_string(),
            add_precommand: "Add a new precommand".to_string(),
            edit_config: "Edit config".to_string(),
            edit_precommand: "Edit precommand".to_string(),
            generate_rsa: "Generate RSA key".to_string(),
            exit: "Exit".to_string(),
            invalid_choice: "Invalid choice".to_string(),
            error: "Error".to_string(),
            success: "Success".to_string(),
            enter_command: "Enter a command to execute before connecting to the host:".to_string(),
            command_added: "Command added successfully".to_string(),
            enter_email: "Enter your email:".to_string(),
            rsa_generated: "RSA key generated successfully".to_string(),
            rsa_failed: "Failed to generate RSA key".to_string(),
            no_precommand: "No precommand found".to_string(),
            choose_command: "Choose a command".to_string(),
            execute_command: "Now execute command: ssh".to_string(),
            no_suitable_editor: "No suitable editor found".to_string(),
            install_editor: "Please install a text editor like VSCode, Vim, or Nano".to_string(),
            invalid_host: "Invalid host".to_string(),
            invalid_port: "Invalid port".to_string(),
            empty_fields: "You can't proceed without filling all the fields".to_string(),
        }
    }
    
    fn chinese_translations() -> Translations {
        Translations {
            menu_title: "菜单".to_string(),
            menu_options: vec![
                "连接".to_string(),
                "执行预命令".to_string(),
                "添加新主机".to_string(),
                "添加新预命令".to_string(),
                "编辑配置".to_string(),
                "编辑预命令".to_string(),
                "生成RSA密钥".to_string(),
                "退出".to_string(),
            ],
            choose_host: "选择主机".to_string(),
            no_hosts: "您没有任何可连接的主机".to_string(),
            add_host: "添加新主机".to_string(),
            add_host_prompt: "输入SSH访问的域名或IP地址:".to_string(),
            user_prompt: "输入SSH访问的用户名:".to_string(),
            port_prompt: "输入SSH访问的端口:".to_string(),
            hostname_prompt: "输入SSH访问的主机名:".to_string(),
            host_added: "主机添加成功".to_string(),
            connect: "连接".to_string(),
            execute_precommand: "执行预命令".to_string(),
            add_precommand: "添加新预命令".to_string(),
            edit_config: "编辑配置".to_string(),
            edit_precommand: "编辑预命令".to_string(),
            generate_rsa: "生成RSA密钥".to_string(),
            exit: "退出".to_string(),
            invalid_choice: "无效选择".to_string(),
            error: "错误".to_string(),
            success: "成功".to_string(),
            enter_command: "输入在连接到主机之前要执行的命令:".to_string(),
            command_added: "命令添加成功".to_string(),
            enter_email: "输入您的邮箱:".to_string(),
            rsa_generated: "RSA密钥生成成功".to_string(),
            rsa_failed: "RSA密钥生成失败".to_string(),
            no_precommand: "未找到预命令".to_string(),
            choose_command: "选择命令".to_string(),
            execute_command: "现在执行命令: ssh".to_string(),
            no_suitable_editor: "未找到合适的编辑器".to_string(),
            install_editor: "请安装文本编辑器，如VSCode、Vim或Nano".to_string(),
            invalid_host: "无效的主机".to_string(),
            invalid_port: "无效的端口".to_string(),
            empty_fields: "您必须填写所有字段才能继续".to_string(),
        }
    }
}

// 全局语言实例
use std::sync::OnceLock;

static LOCALE: OnceLock<Locale> = OnceLock::new();

pub fn get_locale() -> &'static Locale {
    LOCALE.get_or_init(|| Locale::new())
}

pub fn init_locale() {
    let _ = LOCALE.set(Locale::new());
}