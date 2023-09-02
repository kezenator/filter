pub struct Graph
{
    trace_count: usize,
    width: usize,
    height: usize,
    svg: String,
    
}

impl Graph
{
    pub fn new() -> Self
    {
        let trace_count = 0;
        let width = 1500;
        let height = 800;
        let mut svg = format!(r#"<svg version="1.1" width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
            width, height);
        svg.push_str(&format!(r#"<path d="M0 {} L {} {}" stroke="black"/>"#,
            height / 2, width, height / 2));

        Graph { trace_count, width, height, svg }
    }

    pub fn add_trace(&mut self, values: &[f64], range: f64, label: &str, units: &str)
    {
        let color_index = self.trace_count % COLORS.len();
        let color = COLORS[color_index];

        self.svg.push_str(&format!(r#"<rect x="{}" y="{}" width="20" height="20" fill="{}"/>"#,
            10, 10 + 30 * self.trace_count, color));

        self.svg.push_str(&format!(r#"<text x="{}" y="{}" fill="{}" text-anchor="start" dominant-baseline="middle" font-family="monospace" font-weight="bold">"#,
            35, 20 + 30 * self.trace_count, color));
        self.svg.push_str(&format!(r#"{} (±{}{})</text>"#,
            label, range, units));

        self.svg.push_str(r#"<path d=""#);
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

            self.svg.push_str(&format!("{}{} {}", char, x, y));
        }
        self.svg.push_str(&format!(r#"" stroke="{}" stroke-width="3" fill="transparent"/>"#, color));
        self.trace_count += 1;
    }

    pub fn to_svg(mut self) -> String
    {
        self.svg.push_str("</svg>");
        self.svg
    }
}

const COLORS: [&'static str;2] = ["darkseagreen", "coral"];