use regex::Regex;

pub fn include_shader(shader_includes: Vec<&'static str>, shader: &'static str) -> String {
    let mut final_shader = String::new();

    final_shader.push_str(shader);

    let re = Regex::new(r"#include \d").unwrap();

    for capture in re.captures_iter(shader) {
        let include_text: String = capture.get(0).unwrap().as_str().into();
        let include_id_text = include_text.replace("#include ", "");
        let include_id: usize = include_id_text.parse::<usize>().unwrap();
        final_shader = final_shader.replace(&include_text, shader_includes[include_id]);
    }

    final_shader
}
