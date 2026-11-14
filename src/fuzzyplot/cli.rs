use clap::Parser;

// TODO add color to CLI
#[derive(Parser)]
#[command(version, verbatim_doc_comment)]
/** Outputs a fuzzy-plotted graph image of a given equation

Fuzzyplot is a graph-plotting program. Instead of plotting points where
the equation is exactly satisfied, fuzzyplot colors points with an intensity
depending on the difference between the two expressions.
*/
pub struct Cli {
    /** Don't draw axes

    When this flag is used, the grid and axes won't be drawn.
    */
    #[arg(short = 'A', long = "axisless", verbatim_doc_comment)]
    pub no_axes: bool,
    /** Evaluate difference in proportion to magnitude (quotient)

    Divide the difference in the equation by the magnitude of the two
    expressions, in order to counteract the bias toward small values.
    */
    #[arg(short, long, conflicts_with = "plain_diff", verbatim_doc_comment)]
    pub quotient: bool,
    /** Don't draw poles

    By default, points where an equation's difference is infinite are drawn
    in the complement of that equation's colour. When this flag is used,
    only solutions are drawn.
    */
    #[arg(short, long, verbatim_doc_comment)]
    pub no_poles: bool,
    /** Evaluate plain difference

    Like -q, but without dividing by the magnitude of the expressions.
    Sometimes certain graphs work better with just the plain difference.
    */
    #[arg(short, long = "plain", verbatim_doc_comment)]
    pub plain_diff: bool,
    /** Image dimensions [default: 800, 800]

    The width and height must both be greater than zero.
    */
    #[arg(
        short,
        long,
        num_args = 2,
        value_names = ["WIDTH", "HEIGHT"],
        verbatim_doc_comment
    )]
    pub dimensions: Vec<u32>,
    /** View focus on graph [default: 0, 0]

    The center of the view area in graph coordinates.
    */
    #[arg(
        short,
        long,
        num_args = 2,
        value_names = ["X", "Y"],
        allow_negative_numbers = true,
        verbatim_doc_comment
    )]
    pub focus: Vec<f64>,
    /** Grid size [default: 1]

    The size of the interval between the grid lines. Grid lines will not be
    shown if the -A/--axisless flag is given. A grid size of zero will turn
    off the grid lines (but leave the axes).
    */
    #[arg(
        short,
        long,
        default_value = "1",
        hide_default_value = true,
        allow_negative_numbers = true,
        verbatim_doc_comment
    )]
    pub grid_size: f64,
    /**Filename of the new image [default: graph.png]

    The file extension given will determine the format in which the
    image is saved. Currently only {jpeg, png, ico, bmp, tiff} are supported.
    If the file already exists, it will be overwritten unless it is
    write-protected, in which case the fuzzyplot will give an error.
    */
    #[arg(
        short,
        long,
        default_value = "graph.png",
        hide_default_value = true,
        verbatim_doc_comment
    )]
    pub outfile: std::path::PathBuf,
    /** The sharpness of the graph [default: 0]

    This is a pretty arbitrary parameter, but 0 is reasonable for most cases,
    higher than that is more sharp, lower than that is more fuzzy. Turning down
    the sharpness can sometimes show solutions more clearly, but makes things
    less clear and sometimes reveals false solutions too.
    */
    #[arg(
        short,
        long,
        default_value = "0",
        hide_default_value = true,
        allow_negative_numbers = true,
        verbatim_doc_comment
    )]
    pub sharpness: f64,
    /** Range of theta (t) values considered [default: 0, 0]

    This is a strange one. The range given must be integers. Each integer `n`
    corresponds to a full-circle angle range, where the midpoint of the range
    is nτ (that's a tau). e.g:
    · n = 0  gives [-τ/2 , τ/2 ] (midpoint 0 )
    · n = 1  gives [τ/2  , 3τ/2] (midpoint τ )
    · n = -1 gives [-3τ/2, -τ/2] (midpoint -τ)
    · ...
    The range given to this argument is the range of n, inclusive.
    */
    #[arg(
        short,
        long,
        num_args = 2,
        value_names = ["MIN", "MAX"],
        allow_negative_numbers = true,
        verbatim_doc_comment
    )]
    pub t_range: Vec<i32>,
    /** Zoom level [default: -1]

    A zoom of `z` sets the "radius" of the view area to 2^(-z).
    e.g. `-z -1` with a square image shows from (-2, -2) to (2, 2). This means
    positive numbers zoom in, and negative numbers zoom out. For non-square
    images, the shorter distance is taken to be the radius, and the longer
    side scales in proportion.
    */
    #[arg(
        short,
        long,
        default_value = "-1",
        hide_default_value = true,
        allow_negative_numbers = true,
        verbatim_doc_comment
    )]
    pub zoom: f64,
    /** Equation(s) to plot (maximum 3)

    Up to 3 equations of the form "<expression>=<expression>", where each
    expression involves any of {x, y, r, t (short for theta)}. Whitespace is
    generally optional in obvious places. Available symbols are:
    · Operators [ +, -, *, /, ^ ] - e.g. (-x)^3
    · Trig [sin, cos, tan, asin, acos, atan] - e.g. sin(x)
    · Misc - sqrt(n), abs(n), log(n, base), gamma(n)
    · Constants [e, i, pi, tau] - e.g. 2 + 3i
    */
    // allows e.g. "-r=t"
    #[arg(
        required = true,
        num_args = 1..=3,
        verbatim_doc_comment
    )]
    pub equations: Vec<String>,
}
