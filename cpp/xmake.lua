add_rules("mode.debug", "mode.release")

add_includedirs("includes")

set_languages("c++20")
set_toolchains("gcc", {cxxflags = "-std=c++20"})

target("shs")
    set_kind("binary")
    add_files("src/*.cpp")


target("clean")
    set_kind("phony")
    after_build(function ()
        os.rm("./build", {recursive = true})
    end)
