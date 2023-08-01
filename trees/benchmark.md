# Benchmark
## Small
| Test              | Gears                      | Go           | Ratio                               |
| :---------------- | :------------------------- | :----------  | :---------------------------------- |
| Query miss (fast) |                            | 589ns           |      |
| Query miss (slow) |  5.016µs | 1.617µs           | <mark style="background-color: red">&nbsp;3.1&nbsp;</mark>                                     |
| Query hit (fast)  |                            | 61ns           |       |
| Query hit (slow)  |  5.112µs  | 2.96µs           | <mark style="background-color: red">&nbsp;1.7&nbsp;</mark>                                     |
| Iter (fast)       |                            | 505.801µs           |            |
| Iter (slow)       | 1.06313ms        | 2.181263ms           | <mark style="background-color: green">&nbsp;0.5&nbsp;</mark>                                     |
| Update            |  35.266µs     | 29.918µs           | <mark style="background-color: red">&nbsp;1.2&nbsp;</mark>          |
| Run Blocks        |  10.764803ms | 7.348834ms           | <mark style="background-color: red">&nbsp;1.5&nbsp;</mark>      |
## Medium
| Test              | Gears                      | Go           | Ratio                               |
| :---------------- | :------------------------- | :----------  | :---------------------------------- |
| Query miss (fast) |                            | 2.34µs           |      |
| Query miss (slow) |  19.178µs | 9.099µs           | <mark style="background-color: red">&nbsp;2.1&nbsp;</mark>                                     |
| Query hit (fast)  |                            | 406ns           |       |
| Query hit (slow)  |  19.992µs  | 12.909µs           | <mark style="background-color: red">&nbsp;1.5&nbsp;</mark>                                     |
| Iter (fast)       |                            | 41.978635ms           |            |
| Iter (slow)       | 279.375714ms        | 964.896104ms           | <mark style="background-color: green">&nbsp;0.3&nbsp;</mark>                                     |
| Update            |  139.529µs     | 116.014µs           | <mark style="background-color: red">&nbsp;1.2&nbsp;</mark>          |
| Run Blocks        |  17.022357ms | 16.063524ms           | <mark style="background-color: red">&nbsp;1.1&nbsp;</mark>      |
## Large
| Test              | Gears                      | Go           | Ratio                               |
| :---------------- | :------------------------- | :----------  | :---------------------------------- |
| Query miss (fast) |                            | 5.139µs           |      |
| Query miss (slow) |  117.3µs | 17.639µs         | <mark style="background-color: red">&nbsp;6.7&nbsp;</mark>                                     |
| Query hit (fast)  |                            | 5.339µs           |       |
| Query hit (slow)  |  130.857µs  | 23.944µs           | <mark style="background-color: red">&nbsp;5.5&nbsp;</mark>                                     |
| Iter (fast)       |                            | 651.533418ms           |            |
| Iter (slow)       | 14.441936665s        | 8.784634345s           | <mark style="background-color: red">&nbsp;1.6&nbsp;</mark>                                     |
| Update            |  287.821µs     | 242.246µs           | <mark style="background-color: red">&nbsp;1.2&nbsp;</mark>          |
| Run Blocks        |  24.938731ms | 54.795291ms           | <mark style="background-color: green">&nbsp;0.5&nbsp;</mark>      |
