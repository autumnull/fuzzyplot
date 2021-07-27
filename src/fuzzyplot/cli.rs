use structopt::StructOpt;

// TODO add center, accuracy, grid_size to CLI
#[derive(StructOpt)]
#[structopt(
    setting(clap::AppSettings::AllowNegativeNumbers),
    verbatim_doc_comment
)]
/** Outputs a fuzzy-plotted graph image of a given equation

Fuzzyplot is a graph-plotting program. Instead of plotting points where
the equation is exactly satisfied, fuzzyplot colors points with an intensity
depending on the difference between the two expressions.
*/
pub struct Cli {
    /** Don't draw axes

    When this flag is used, the grid and axes won't be drawn.
    */
    #[structopt(short = "A", long = "axisless", verbatim_doc_comment)]
    pub no_axes: bool,
    /** Evaluate plain difference, not proportional to magnitude

    By default, fuzzyplot divides the difference in the equation by the
    magnitude of the two expressions, in order to counteract the bias toward
    small values. Sometimes certain graphs work better with just plain
    difference, without the division. This flag sets that mode.
    */
    #[structopt(short, long = "plain", verbatim_doc_comment)]
    pub plain_diff: bool,
    /**Filename of the new image [default: graph.png]

    The file extension given will determine the format in which the
    image is saved. Currently only {jpeg, png, ico, bmp, tiff} are supported.
    If the file already exists, it will be overwritten unless it is
    write-protected, in which case the fuzzyplot will give an error.
    */
    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = "graph.png",
        set(clap::ArgSettings::HideDefaultValue),
        verbatim_doc_comment
    )]
    pub outfile: std::path::PathBuf,
    /** Image size [default: 800, 800]

    The width and height must both be greater than zero.
    */
    #[structopt(short, long, value_names(&["WIDTH", "HEIGHT"]), verbatim_doc_comment)]
    pub size: Vec<u32>,
    /** Range of theta (t) values considered [default: 0, 0]

    This is a strange one. The range given must be integers. Each integer `n`
    corresponds to a full-circle angle range, where the midpoint of the range
    is nτ (that's tau). e.g:
    · n = 0  gives [-τ/2 , τ/2 ] (midpoint 0 )
    · n = 1  gives [τ/2  , 3τ/2] (midpoint τ )
    · n = -1 gives [-3τ/2, -τ/2] (midpoint -τ)
    · ...
    The range given to this argument is the range of n, inclusive.
    */
    #[structopt(short, long, value_names(&["MIN", "MAX"]), verbatim_doc_comment)]
    pub t_range: Vec<i32>,
    /** Zoom level [default: -1]

    `--zoom z` sets the "radius" of the view area to 2^(-z).
    e.g. `-z -1` with a square image shows from (-2, -2) to (2, 2). This means
    positive numbers zoom in, and negative numbers zoom out. For non-square
    images, the shorter distance is taken to be the radius, and the longer
    side scales in proportion.
    */
    #[structopt(
        short,
        long,
        default_value = "-1",
        set(clap::ArgSettings::HideDefaultValue),
        verbatim_doc_comment
    )]
    pub zoom: f64,
    /** Equation(s) to plot (maximum 3)

    Up to 3 equations of the form "<expression>=<expression>", where each
    expression involves any of {x, y, r, t (short for theta)}. Whitespace is
    generally optional in obvious places. Available symbols are:
    · Operators [ +, -, *, /, ^ ] - e.g. (-x)^3
    · Trig [sin, cos, tan, asin, acos, atan] - e.g. sin(x)
    · Misc - sqrt(n), abs(n), log(n, base)
    · Constants [e, i, pi, tau] - e.g. 2 + 3i
    */
    // allows e.g. "-r=t"
    #[structopt(
        allow_hyphen_values(true),
        required(true),
        max_values(3),
        verbatim_doc_comment
    )]
    pub equations: Vec<String>,
}
