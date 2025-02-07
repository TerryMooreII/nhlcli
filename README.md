# NHL CLI Tool

## Why
Fun side project to learn Rust. I previously build this in node but the NHL changed their API and I had to rewrite it.

## Installation

```
cargo build
```

## Usage


### Show a list of games for yesterday, today, and tomorrow and their scores
```
nhlcli scores
```

### Show current NHL standings
```
nhlcli standings wildcard
nhlcli standings conference
nhlcli standings league
```

### Show current NHL leaders

For skaters:
```
nhlcli leaders points
nhlcli leaders goals
nhlcli leaders assists
nhlcli leaders penalty-minutes
nhlcli leaders toi
nhlcli leaders plus-minus
nhlcli leaders faceoffs
```

For goalies:
```
nhlcli leaders save-percentage
nhlcli leaders goals-against-avg
nhlcli leaders shutouts
nhlcli leaders wins
```

### Show detailed boxscore for a specific game
```
nhlcli boxscores
```

### Show Ovi's goals and how many more he needs to beat Gretzky's record
```
nhlcli ovi
```

## License
MIT License