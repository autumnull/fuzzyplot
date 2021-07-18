# fuzzyplot

### A fuzzier graph plotting method

![](images/heart.png)

![](https://render.githubusercontent.com/render/math?math=%7B%28x%5E2%20%2B%20y%5E2%20-%201%29%7D%5E3%20%3D%20x%5E2y%5E3)

#### How does it work ?

`fuzzyplot` graphs equations of the form ![](https://render.githubusercontent.com/render/math?math=f%28x%2Cy%2Cr%2C%5Ctheta%29%20%3D%20g%28x%2Cy%2Cr%2C%5Ctheta%29). Instead of finding the points where the two sides of the equation are equal (as in traditional plotting methods), `fuzzyplot` shows the points where the two sides of the equation are *very close* to being equal. This is done with a sort of fuzzy effect, where the smaller the difference between the two sides is, the more intense the color at that point will be. Hence the name.

#### What benefits does this give ?

- Shows point-solutions that may be hard to find with other methods
- Makes the graphs look soft and pretty ^\_^

### Pictures !

![](images/heart_axes.png)

![](https://render.githubusercontent.com/render/math?math=%7B%28x%5E2%20%2B%20y%5E2%20-%201%29%7D%5E3%20%3D%20x%5E2y%5E3) with axes (axes can be set on or off).

![](images/trident.png)

![](https://render.githubusercontent.com/render/math?math=x%5Ey%20%3D%20y%5Ex)

`fuzzyplot` uses complex intermediate values, which can reveal hidden point-solutions, as in the above example, and the one below

![](images/fishhook.png)

![](https://render.githubusercontent.com/render/math?math=y%20%3D%20x%5Ex)
