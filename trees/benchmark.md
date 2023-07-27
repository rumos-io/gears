# Benchmark

## Small

| Test              | Gears       | Go           | Multiple                                        |
| :---------------- | :------     | :----------  | :---------------------------------------------- |
| Query miss (fast) |  5.4466 µs  | 0.5894 µs    | <mark style="background-color: red">9.2</mark>  |
| Query miss (slow) |             | 1.617 µs     |                                                 |
| Query hit (fast)  |  5.6332 µs  | 61.31 ns     | <mark style="background-color: red">91</mark>   |
| Query hit (slow)  |             | 2.960 µs     |                                                 |
| Iter (fast)       |  1.2397 ms  | 0.505801 ms  | <mark style="background-color: red">2.5</mark>  |
| Iter (slow)       |             | 2.181263 ms  |                                                 |
| Update            |  46.064 µs  | 29.918 µs    | <mark style="background-color: red">1.5</mark>  |
| Run Blocks        |  12.215 ms  | 7.348834 ms  | <mark style="background-color: red">1.7</mark>  |

## Medium

| Test              | Gears       | Go             | Multiple                                        |
| :---------------- | :------     | :------------  | :---------------------------------------------- |
| Query miss (fast) |   19.683 µs | 2.340 µs       | <mark style="background-color: red">8.4</mark>  |
| Query miss (slow) |             | 9.099 µs       |                                                 |
| Query hit (fast)  |   21.266 µs | 406.9 ns       | <mark style="background-color: red">52.3</mark> |
| Query hit (slow)  |             | 12.909 µs      |                                                 |
| Iter (fast)       |   299.24 ms | 41.978635 ms   | <mark style="background-color: red">7.1</mark>  |
| Iter (slow)       |             | 964.896104 ms  |                                                 |
| Update            |   168.92 µs | 116.014 µs     | <mark style="background-color: red">1.5</mark>  |
| Run Blocks        |   18.922 ms | 16.063524 ms   | <mark style="background-color: red">1.2</mark>  |

## Large

| Test              | Gears       | Go             | Multiple                                          |
| :---------------- | :------     | :------------  | :------------------------------------------------ |
| Query miss (fast) |   114.15 µs | 5.139 µs       | <mark style="background-color: red">22.2</mark>   |
| Query miss (slow) |             | 1.496849 ms    |                                                   |
| Query hit (fast)  |   137.63 µs | 5.339 µs       | <mark style="background-color: red">25.8</mark>   |
| Query hit (slow)  |             | 23.944 µs      |                                                   |
| Iter (fast)       |   14.717 s  | 651.533418 ms  | <mark style="background-color: red">22.6</mark>     |
| Iter (slow)       |             | 8.784634345 s  |                                                   |
| Update            |   248.42 µs | 242.246 µs     | <mark style="background-color: orange">1.0</mark> |
| Run Blocks        |   28.787 ms | 54.795291 ms   | <mark style="background-color: green">0.5</mark>  |



