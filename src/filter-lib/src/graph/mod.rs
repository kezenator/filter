pub struct Graph
{
    width: usize,
    height: usize,
    svg: String
}

impl Graph
{
    pub fn new() -> Self
    {
        let width = 1500;
        let height = 800;
        let svg = format!(r#"<svg version="1.1" width="{}" height="{}" xmlns="http://www.w3.org/2000/svg"><path d="M0 400 L 1500 400" stroke="black"/>"#,
            width, height);

        Graph { width, height, svg }
    }

    pub fn add_trace(&mut self, values: &[f64], range: f64)
    {
        self.svg += r#"<path d=""#;
        for (i, val) in values.iter().enumerate()
        {
            let char = match i
            {
                0 => 'M',
                _ => 'L',
            };

            let width = self.width as f64;
            let height = self.height as f64;
            let i = i as f64;
            let len = values.len() as f64;

            let x = width * i / len;
            let y = (height * 0.5) - (height * 0.5 * val / range);

            self.svg += &format!("{}{} {}", char, x, y);
        }
        self.svg += r#"" fill="transparent" stroke="black"/>"#;
    }

    pub fn to_svg(self) -> String
    {
        self.svg + "</svg>"
    }
}