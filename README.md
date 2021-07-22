# fuzzyplot :heart:

### A fuzzier graph plotting method

`fuzzyplot "r=1-sin(t)" "r=1/(1-sin(t))" "r=1" -Ap -z -2`

![](images/circle_inversion.png)

![](https://render.githubusercontent.com/render/math?math=%5Cbegin%7Bcases%7D%0Ar%20%3D%201-%5Csin%7B%5Ctheta%7D%5C%5C%0Ar%20%3D%20%7B%281-%5Csin%7B%5Ctheta%7D%29%7D%5E%7B-1%7D%5C%5C%0Ar%20%3D%201%5C%5C%0A%5Cend%7Bcases%7D)

#### How does it work ?

`fuzzyplot` graphs equations of the form ![](https://render.githubusercontent.com/render/math?math=f%28x%2Cy%2Cr%2C%5Ctheta%29%20%3D%20g%28x%2Cy%2Cr%2C%5Ctheta%29). Instead of finding the points where the two sides of the equation are equal (as in traditional plotting methods), `fuzzyplot` shows the points where the two sides of the equation are *very close* to being equal. This is done with a sort of fuzzy effect, where the smaller the difference between the two sides is, the more intense the color at that point will be. Hence the name.

#### What benefits does this give ?

- Shows point-solutions that may be hard to find with other methods
- Makes the graphs look soft and pretty \^\_\^

### Pictures !

Axes can be turned off with the `-A`/`--axisless` flag

`fuzzyplot "(x^2 + y^2 - 1)^3 = x^2 y^3 --axisless"`

![](images/heart.png)

![](https://render.githubusercontent.com/render/math?math=%7B%28x%5E2%20%2B%20y%5E2%20-%201%29%7D%5E3%20%3D%20x%5E2y%5E3)

The `-z`/`--zoom` option lets you set the zoom level. Negative numbers mean zoom out.

`fuzzyplot "x^y = y^x" --zoom -3`

![](images/trident.png)

![](https://render.githubusercontent.com/render/math?math=x%5Ey%20%3D%20y%5Ex)

`fuzzyplot` uses complex intermediate values, which can reveal hidden point-solutions, as in the above example, and the one below

`fuzzyplot "y = x^x" --zoom -3`

![](images/fishhook.png)

![](https://render.githubusercontent.com/render/math?math=y%20%3D%20x%5Ex)

By default, `fuzzyplot` divides the difference in the equation by the magnitude of the two expressions, in order to counteract the bias toward small values. Sometimes certain graphs work better with just plain difference, without the division. This mode can be set with the `-p`/`--plain` flag.

`fuzzyplot "r = 3/2 (1 - sin(t))" --zoom -2 --plain`

![](images/cardioid.png)

![](https://render.githubusercontent.com/render/math?math=r%3D%5Cfrac%7B3%7D%7B2%7D%281-sin%7B%5Ctheta%7D%29)

`fuzzyplot` supports up to 3 equations per image, passed as arguments one after the other. When only 1 equation is given, it is colored red, but if 2 or 3 are given, they are colored cyan, magenta, yellow, in that order. The colors mix together like colored filters.

`fuzzyplot "±r=sin(t)" "±r=sin(t+2pi/3)" "±r=sin(t+4pi/3)" -A`

![](images/flower.png)

![](https://render.githubusercontent.com/render/math?math=%5Cbegin%7Bcases%7D%0Ar%3D%5Cpm%5Csin%7Bt%7D%5C%5C%0Ar%3D%5Cpm%5Csin%7B%28t%2B2%5Cpi%2F3%29%7D%5C%5C%0Ar%3D%5Cpm%5Csin%7B%28t%2B4%5Cpi%2F3%29%7D%5C%5C%0A%5Cend%7Bcases%7D)
