# Relic Emoji Plan

Display relics as emoji icons in a top bar, below the player HP/energy/block bars.

## Emoji Assignments

| Relic | Emoji | Rationale |
|---|---|---|
| Strawberry | 🍓 | Literal |
| Pear | 🍐 | Literal |
| Mango | 🥭 | Literal |
| OldCoin | 🪙 | Literal |
| Whetstone | 🪨 | A stone used to sharpen blades |
| WarPaint | 🎨 | Paint |
| BurningBlood | 🔥 | Burning |
| BlackBlood | 🖤 | Black heart |
| Anchor | ⚓ | Literal |
| Vajra | 🔱 | Trident — Hindu/Buddhist divine weapon symbol |
| Lantern | 🏮 | Lantern |
| BloodVial | 🩸 | Blood drop |
| BagOfMarbles | 🎱 | Eight ball — round toy |
| RedMask | 😶 | Mask |
| FestivePopper | 🎉 | Party popper |
| Pantograph | 📐 | Drawing tool |
| BagOfPreparation | 🎒 | Bag |
| MercuryHourglass | ⏳ | Hourglass |
| CaptainsWheel | ⚙️ | Steering wheel / gear |
| Chandelier | 💡 | Light fixture |
| Candelabra | 🕯️ | Candle |
| HornCleat | 🪝 | Hook/cleat |
| HappyFlower | 🌸 | Flower |
| Pendulum | 🕰️ | Clock with pendulum |
| StoneCalendar | 📅 | Calendar |
| Orichalcum | 🟠 | Orange circle (mythical orange-gold metal) |
| CloakClasp | 🪆 | Clasp / brooch feel |
| RegalPillow | 🛏️ | Pillow |
| Nunchaku | 🥋 | Martial arts |
| OrnamentalFan | 🪭 | Folding fan |
| Kunai | 🗡️ | Dagger |
| Shuriken | ⭐ | Star shape |
| Kusarigama | ⛓️ | Chain weapon |
| LetterOpener | ✉️ | Letter |
| TuningFork | 🎵 | Musical / vibration |
| GremlinHorn | 📯 | Horn |
| Pocketwatch | ⌚ | Watch |

## Rendering

- Render as a single line of emoji after the player stats bar in both TUI and plain-text renderers.
- Each relic is one emoji with no label — hover/cursor tooltip can show the name in the TUI.
- If the player has no relics, the bar is omitted.
- Example: `⚓ 🔥 🍓 🎒 ⌚`
