use structopt::StructOpt;

// TODO add more detailed help messages
#[derive(StructOpt)]
#[structopt(setting(clap::AppSettings::AllowNegativeNumbers))]
/// Outputs a fuzzy-plotted graph image of a given equation
pub struct Cli {
    /// Don't draw axes
    #[structopt(short = "A", long = "axisless")]
    pub no_axes: bool,
    /// Evaluate plain difference, not proportional to magnitude
    #[structopt(short, long = "plain")]
    pub plain_diff: bool,
    /// Equation(s) to plot (maximum 3)
    #[structopt(allow_hyphen_values(true), required(true), max_values(3))] // allows e.g. "-r=t"
    pub equations: Vec<String>,
    /// Filename of the new image
    #[structopt(short, long, parse(from_os_str), default_value = "graph.png")]
    pub outfile: std::path::PathBuf,
    /// Zoom level  
    #[structopt(short, long, default_value = "-1")]
    pub zoom: f64,
    /// Image size
    #[structopt(short, long, value_names(&["WIDTH", "HEIGHT"]))]
    pub size: Vec<u32>,
    /// Range of theta (t) values considered
    #[structopt(short, long, value_names(&["MIN", "MAX"]))]
    pub t_range: Vec<i32>,
}
