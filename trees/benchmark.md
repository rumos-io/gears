# Benchmark
## Small
| Test              | Gears                      | Go           | Ratio                               |
| :---------------- | :------------------------- | :----------  | :---------------------------------- |
| Query miss (fast) | 5.016µs  | 589ns           | <mark style="background-color: red">&nbsp;8.5&nbsp;</mark>      |
| Query miss (slow) |                            | 1.617µs           |                                     |
| Query hit (fast)  |  5.112µs  | 61ns           | <mark style="background-color: red">&nbsp;83.8&nbsp;</mark>       |
| Query hit (slow)  |                            | 2.96µs           |                                     |
| Iter (fast)       |  1.06313ms       | 505.801µs           | <mark style="background-color: red">&nbsp;2.1&nbsp;</mark>            |
| Iter (slow)       |                            | 2.181263ms           |                                     |
| Update            |  35.266µs     | 29.918µs           | <mark style="background-color: red">&nbsp;1.2&nbsp;</mark>          |
| Run Blocks        |  10.764803ms | 7.348834ms           | <mark style="background-color: red">&nbsp;1.5&nbsp;</mark>      |
## Medium
| Test              | Gears                      | Go           | Ratio                               |
| :---------------- | :------------------------- | :----------  | :---------------------------------- |
| Query miss (fast) | 19.178µs  | 2.34µs           | <mark style="background-color: red">&nbsp;8.2&nbsp;</mark>      |
| Query miss (slow) |                            | 9.099µs           |                                     |
| Query hit (fast)  |  19.992µs  | 406ns           | <mark style="background-color: red">&nbsp;49.2&nbsp;</mark>       |
| Query hit (slow)  |                            | 12.909µs           |                                     |
| Iter (fast)       |  279.375714ms       | 41.978635ms           | <mark style="background-color: red">&nbsp;6.7&nbsp;</mark>            |
| Iter (slow)       |                            | 964.896104ms           |                                     |
| Update            |  139.529µs     | 116.014µs           | <mark style="background-color: red">&nbsp;1.2&nbsp;</mark>          |
| Run Blocks        |  17.022357ms | 16.063524ms           | <mark style="background-color: red">&nbsp;1.1&nbsp;</mark>      |
## Large
| Test              | Gears                      | Go           | Ratio                               |
| :---------------- | :------------------------- | :----------  | :---------------------------------- |
| Query miss (fast) | 117.3µs  | 5.139µs           | <mark style="background-color: red">&nbsp;22.8&nbsp;</mark>      |
| Query miss (slow) |                            | 1.496849ms           |                                     |
| Query hit (fast)  |  130.857µs  | 5.339µs           | <mark style="background-color: red">&nbsp;24.5&nbsp;</mark>       |
| Query hit (slow)  |                            | 23.944µs           |                                     |
| Iter (fast)       |  14.441936665s       | 651.533418ms           | <mark style="background-color: red">&nbsp;22.2&nbsp;</mark>            |
| Iter (slow)       |                            | 8.784634345s           |                                     |
| Update            |  287.821µs     | 242.246µs           | <mark style="background-color: red">&nbsp;1.2&nbsp;</mark>          |
| Run Blocks        |  24.938731ms | 54.795291ms           | <mark style="background-color: green">&nbsp;0.5&nbsp;</mark>      |
