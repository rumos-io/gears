# Benchmark
## Small
| Test              | Gears                      | Go           | Ratio                               |
| :---------------- | :------------------------- | :----------  | :---------------------------------- |
| Query miss (fast) |                            | 589ns           |      |
| Query miss (slow) |  2.916µs | 1.617µs           | <mark style="background-color: red">&nbsp;1.8&nbsp;</mark>                                     |
| Query hit (fast)  |                            | 61ns           |       |
| Query hit (slow)  |  3.483µs  | 2.96µs           | <mark style="background-color: red">&nbsp;1.2&nbsp;</mark>                                     |
| Iter (fast)       |                            | 505.801µs           |            |
| Iter (slow)       | 1.989812ms        | 2.181263ms           | <mark style="background-color: green">&nbsp;0.9&nbsp;</mark>                                     |
| Update            |  42.412µs     | 29.918µs           | <mark style="background-color: red">&nbsp;1.4&nbsp;</mark>          |
| Run Blocks        |  12.724527ms | 7.348834ms           | <mark style="background-color: red">&nbsp;1.7&nbsp;</mark>      |
## Medium
| Test              | Gears                      | Go           | Ratio                               |
| :---------------- | :------------------------- | :----------  | :---------------------------------- |
| Query miss (fast) |                            | 2.34µs           |      |
| Query miss (slow) |  8.957µs | 9.099µs           | <mark style="background-color: green">&nbsp;1.0&nbsp;</mark>                                     |
| Query hit (fast)  |                            | 406ns           |       |
| Query hit (slow)  |  11.174µs  | 12.909µs           | <mark style="background-color: green">&nbsp;0.9&nbsp;</mark>                                     |
| Iter (fast)       |                            | 41.978635ms           |            |
| Iter (slow)       | 446.108829ms        | 964.896104ms           | <mark style="background-color: green">&nbsp;0.5&nbsp;</mark>                                     |
| Update            |  204.168µs     | 116.014µs           | <mark style="background-color: red">&nbsp;1.8&nbsp;</mark>          |
| Run Blocks        |  20.277598ms | 16.063524ms           | <mark style="background-color: red">&nbsp;1.3&nbsp;</mark>      |
## Large
| Test              | Gears                      | Go           | Ratio                               |
| :---------------- | :------------------------- | :----------  | :---------------------------------- |
| Query miss (fast) |                            | 5.139µs           |      |
| Query miss (slow) |  28.28µs | 17.639µs           | <mark style="background-color: red">&nbsp;1.6&nbsp;</mark>                                     |
| Query hit (fast)  |                            | 5.339µs           |       |
| Query hit (slow)  |  33.601µs  | 23.944µs           | <mark style="background-color: red">&nbsp;1.4&nbsp;</mark>                                     |
| Iter (fast)       |                            | 651.533418ms           |            |
| Iter (slow)       | 8.713232392s        | 8.784634345s           | <mark style="background-color: green">&nbsp;1.0&nbsp;</mark>                                     |
| Update            |  390.865µs     | 242.246µs           | <mark style="background-color: red">&nbsp;1.6&nbsp;</mark>          |
| Run Blocks        |  28.292003ms | 54.795291ms           | <mark style="background-color: green">&nbsp;0.5&nbsp;</mark>      |
