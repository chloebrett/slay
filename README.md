# slay

A Slay the Spire ⚔️ clone that lives in your terminal 👾, written in Rust 🦀

![1-day-demo](docs/1-day-demo.gif)

Interactive TUI with:

- live event log
- enemy intents
- HP bars
- hand of cards

Falls back to plain text automatically if you pipe stdin or run it in a script.

The demo you see above was built in just one day! This was an experiment for me to play around with Claude Code, including a modified version of [citypaul's configuration](https://github.com/citypaul/.dotfiles/blob/main/claude) which pushes for TDD and good engineering practices.

## What's in

**Combat**

- [x] Turn-based combat — draw 5, spend energy, discard, repeat
- [x] Statuses: Block, Vulnerable, Weak, Poison, Strength, Dexterity, Entangle
- [x] 27 Ironclad cards, most with + upgrades (Strike, Bash, Inflame, Impervious, Body Slam, Cleave, Bludgeon, ...)
- [x] 10 enemies, each with their own move patterns and probabilistic AI (Louse, Red/Green Louse, Fungibeast, Cultist, Jaw Worm, Small Spike Slime, Small Acid Slime, Blue Slaver, Red Slaver)
- [x] 9 potions (Fire, Explosive, Block, Strength, Swift, Fear, Weak, Blood, Energy) — carry up to 3, discard anytime

**Run structure**

- [x] Linear map: Combat x 3, Rest Site, Boss
- [x] Card rewards after combat (pick 1 of 3)
- [x] Rest site: heal 30% HP or upgrade a card
- [x] Gold drops, persistent across floors
- [x] 28 relics with real effects (Burning Blood, Orichalcum, Mercury Hourglass, Bag of Preparation, ...)

## How to run

```
cargo run                              # ratatui TUI (default when stdout is a TTY)
cargo run -- --plain                   # plain text, reads commands from stdin
cargo run -- --script path/to/file     # run a script of newline-separated commands
cargo run -- --debug                   # unlocks win / skip / add / relic / potion commands
```

Commands:
— type a number to play a card

- `e` to end your turn
- `use 1` to use your first potion
- `z`/`x`/`c` peek at your draw, discard, and exhaust piles.

## What's next

The big things that would make runs feel more like the real game:

- Branching map — right now the map is a fixed linear path. Soon: a proper graph with Combat / Elite / Rest / Merchant / Event / Boss nodes, where you choose your route.
- More cards — the Ironclad set has a lot more interesting decisions to make (exhaust synergies, multi-hit attacks, more powers). The full card list is mapped out in `plans/ironclad_cards.json`.
- More relics — 28 are live, another ~60 are planned. The remaining tiers need card-play counters, HP-change hooks, and a couple of new status types (Thorns, Plating).
- Shop — buy and remove cards, buy relics and potions.
- TUI polish — colour-coded statuses, card cost colouring, mouse support.

## Legal disclaimer

I built this because I love the original game so much. Slay the Spire is a registered trademark by Mega Crit, LLC. Please support the developers of this amazing game on Steam: https://store.steampowered.com/app/646570/Slay_the_Spire/
