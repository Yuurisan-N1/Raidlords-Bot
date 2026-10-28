// SiegeSim: deterministic battle simulation shared by the browser (live play) and the server (replay check).
// Rules for determinism: integer world coordinates (milli-units), fixed 30 ticks/s, no Math.random / Date,
// no transcendental Math functions (sin/cos/atan2/hypot); only + - * / and an exact integer sqrt.
(function (root, factory) {
  if (typeof module === 'object' && module.exports) module.exports = factory();
  else root.SiegeSim = factory();
})(this, function () {
  'use strict';
  const TPS = 30, MU = 1000;
  const MAX_TICKS = 180 * TPS;                       // 3 minute battle
  const S3 = 1732;                                    // sqrt(3) * 1000, hex row height

  // ---------- integer helpers ----------
  function isqrt(n) {
    if (n <= 0) return 0;
    let r = Math.floor(Math.sqrt(n));
    while (r * r > n) r--;
    while ((r + 1) * (r + 1) <= n) r++;
    return r;
  }
  const idist = (ax, ay, bx, by) => isqrt((ax - bx) * (ax - bx) + (ay - by) * (ay - by));
  const tdiv = (a, b) => Math.trunc(a / b);
  const hx = (q, r) => [2000 * q + 1000 * r, S3 * r];
  const hdist = (q, r) => Math.max(Math.abs(q), Math.abs(r), Math.abs(q + r));
  function toHex(x, y) {
    const r = y / S3, q = x / 2000 - r / 2, s = -q - r;
    let rq = Math.round(q), rr = Math.round(r); const rs = Math.round(s);
    const dq = Math.abs(rq - q), dr = Math.abs(rr - r), ds = Math.abs(rs - s);
    if (dq > dr && dq > ds) rq = -rr - rs; else if (dr > ds) rr = -rq - rs;
    return [rq, rr];
  }
  const DIRS = [[1, 0], [0, 1], [-1, 1], [-1, 0], [0, -1], [1, -1]];
  const ARM = { 0: [1000, 0], 60: [500, 866], 120: [-500, 866], 180: [-1000, 0], 240: [-500, -866], 300: [500, -866] };   // edge midpoints of a hex, mu
  function ring(n) { let q = 0, r = -n; const out = []; for (let i = 0; i < 6; i++) for (let k = 0; k < n; k++) { out.push([q, r]); q += DIRS[i][0]; r += DIRS[i][1]; } return out; }
  const mod = (a, n) => ((a % n) + n) % n;
  function rng(seed) { return () => { seed |= 0; seed = seed + 0x6D2B79F5 | 0; let t = Math.imul(seed ^ seed >>> 15, 1 | seed); t = t + Math.imul(t ^ t >>> 7, 61 | t) ^ t; return ((t ^ t >>> 14) >>> 0) / 4294967296; }; }

  // ---------- rules (ints: mu, ticks) ----------
  // Clash level-1 numbers. 1 Clash tile = 600 mu on our map. Times in ticks (30/s).
  // speed: mu per tick = the Clash movement speed x 2.5 (one factor for every troop, so they keep Clash's order and gaps)
  // Balance pass 2026-10-26 (owner's decision, from ~140,000 simulated battles): PEKKA hp 3000->2200 dmg 468->380 (it 3-starred every
  // Keep 8 village), Dragon hp 1900->1700, Hog Rider hp 270->340, Balloon hp 150->240 (200, then 240 after the re-check) dmg 75->100 speed 25->32, Goblin hp 25->36
  // (these four were near useless alone). Barbarian Chief: level 1 hp 722->1445 and the table's level-1 entries fixed (they were half,
  // so level 2 doubled the damage); level 2 and up are unchanged in hp and back to the intended damage.
  const TROOPS = {
    barbarian:   { hp: 45,  speed: 45, range: 240,  dmg: 9,  atk: 30, hitAt: 16, pref: 'any', housing: 1 },
    archer:      { hp: 22,  speed: 60, range: 2100, dmg: 8,  draw: 15, release: 15, ranged: 'arrow', pref: 'any', housing: 1 },
    giant:       { hp: 400, speed: 30, range: 600,  dmg: 24, atk: 60, hitAt: 30, pref: 'def', housing: 5 },
    wallbreaker: { hp: 20,  speed: 60, range: 300,  boom: 400, boomOther: 10, boomR: 1200, pref: 'wall', housing: 2 },
    wizard:      { hp: 75,  speed: 40, range: 1800, dmg: 75, atk: 45, hitAt: 20, ranged: 'orb', splash: 180, pref: 'any', housing: 4 },
    goblin:      { hp: 36,  speed: 80, range: 240,  dmg: 11, atk: 30, hitAt: 15, pref: 'res', resMult: 2, housing: 1 },
    pekka:       { hp: 2200, speed: 40, range: 480, dmg: 380, atk: 54, hitAt: 27, pref: 'any', housing: 25 },
    // flies (cannon and mortar cannot hit it), heals every friendly ground troop near its patient
    healer:      { hp: 500, speed: 40, range: 3000, heal: 25, atk: 21, healSplash: 900, air: true, pref: 'heal', housing: 14 },
    // flyers: over the walls; only Air Defense, archers, wizards and the like can hit them
    balloon:     { hp: 240, speed: 32, range: 300, dmg: 100, atk: 90, hitAt: 45, ranged: 'bomb', splash: 720, air: true, pref: 'def', death: { dmg: 25, r: 720 }, housing: 5 },
    dragon:      { hp: 1700, speed: 40, range: 1800, dmg: 175, atk: 37, hitAt: 18, ranged: 'fire', splash: 180, air: true, pref: 'any', housing: 20 },
    babydragon:  { hp: 1200, speed: 50, range: 1650, dmg: 75, atk: 30, hitAt: 15, ranged: 'fire', splash: 180, air: true, pref: 'any', housing: 10 },
    minion:      { hp: 58, speed: 80, range: 1650, dmg: 38, atk: 30, hitAt: 15, ranged: 'orb', air: true, pref: 'any', housing: 2 },
    // dark barracks / later barracks. burrow: underground while moving (untargetable, straight line)
    miner:       { hp: 550, speed: 80, range: 300, dmg: 136, atk: 51, hitAt: 26, pref: 'any', burrow: true, housing: 6 },
    hogrider:    { hp: 340, speed: 60, range: 360, dmg: 60, atk: 30, hitAt: 15, pref: 'def', jump: true, housing: 5 },
    valkyrie:    { hp: 750, speed: 60, range: 300, dmg: 169, atk: 54, hitAt: 30, pref: 'any', spin: 600, housing: 8 },
    golem:       { hp: 5100, speed: 30, range: 600, dmg: 84, atk: 72, hitAt: 36, pref: 'def', death: { dmg: 350, r: 900, spawn: 'golemite', n: 2 }, housing: 30 },
    witch:       { hp: 300, speed: 30, range: 2400, dmg: 70, atk: 21, hitAt: 10, ranged: 'orb', pref: 'any', summon: { type: 'skeleton', n: 4, every: 210, max: 6 }, housing: 12 },
    bowler:      { hp: 325, speed: 35, range: 1800, dmg: 132, atk: 66, hitAt: 33, ranged: 'rock', bounce: 1500, pref: 'any', housing: 6 },
    icegolem:    { hp: 2600, speed: 30, range: 600, dmg: 48, atk: 60, hitAt: 30, pref: 'def', death: { freeze: 120, r: 3300 }, housing: 15 },
    headhunter:  { hp: 360, speed: 80, range: 1800, dmg: 63, atk: 18, hitAt: 9, ranged: 'dagger', pref: 'any', housing: 6 },
    appwarden:   { hp: 1500, speed: 50, range: 3000, dmg: 153, atk: 27, hitAt: 13, ranged: 'orb', pref: 'any', aura: 4200, housing: 20 },
    druid:       { hp: 1300, speed: 60, range: 3000, heal: 80, healN: 4, atk: 30, pref: 'heal', housing: 16 },
    yeti:        { hp: 2900, speed: 30, range: 480, dmg: 230, atk: 30, hitAt: 15, pref: 'def', mites: { type: 'yetimite', n: 8 }, housing: 18 },
    etitan:      { hp: 7200, speed: 40, range: 750, dmg: 270, atk: 45, hitAt: 22, pref: 'any', zap: { r: 2100, dmg: 30, every: 12 }, housing: 32 },
    rootrider:   { hp: 6200, speed: 30, range: 480, dmg: 209, atk: 66, hitAt: 33, wallMult: 20, pref: 'def', housing: 20 },
    thrower:     { hp: 2200, speed: 40, range: 3600, dmg: 475, atk: 75, hitAt: 37, ranged: 'spear', pref: 'any', housing: 16 },
    meteorgolem: { hp: 14500, speed: 30, range: 3000, dmg: 480, atk: 30, hitAt: 15, ranged: 'fire', splash: 600, wallMult: 5, pref: 'any', housing: 40 },
    ruinwitch:   { hp: 2300, speed: 30, range: 480, dmg: 50, atk: 30, hitAt: 15, pref: 'any', summon: { type: 'ruinknight', n: 2, every: 150, max: 10 }, housing: 26 },
    // summoned only (never trained)
    golemite:    { hp: 1020, speed: 30, range: 600, dmg: 17, atk: 72, hitAt: 36, pref: 'def', summoned: true, housing: 0 },
    skeleton:    { hp: 30, speed: 60, range: 240, dmg: 25, atk: 30, hitAt: 15, pref: 'any', summoned: true, housing: 0 },
    yetimite:    { hp: 104, speed: 80, range: 240, dmg: 39, atk: 30, hitAt: 15, wallMult: 4, pref: 'any', summoned: true, housing: 0 },
    ruinknight:  { hp: 300, speed: 55, range: 300, dmg: 40, atk: 30, hitAt: 15, pref: 'any', summoned: true, housing: 0 },
    // heroes: one each, ability used once via an 'ability:<hero>' log command
    bk:          { hp: 1445, speed: 40, range: 600, dmg: 122, atk: 36, hitAt: 18, pref: 'any', hero: true, ability: { heal: 500, rage: 300, dmgUp: 56, spawn: 'barbarian', n: 6 }, housing: 0 },
    aq:          { hp: 580, speed: 60, range: 3000, dmg: 77, atk: 22, hitAt: 11, ranged: 'bolt', pref: 'any', hero: true, ability: { heal: 150, hide: 114, dmgUp: 80 }, housing: 0 },
    gw:          { hp: 850, speed: 40, range: 4200, dmg: 139, atk: 54, hitAt: 27, ranged: 'orb', pref: 'any', hero: true, aura: 4200, ability: { invuln: 105, r: 4200 }, housing: 0 },
    // the player's own champion (one per player): level 1-40 from play, Valor rank 0-20 from tokens (see champStat); War Cry rallies nearby troops
    champion:    { hp: 480, speed: 45, range: 600, dmg: 36, atk: 30, hitAt: 15, pref: 'any', hero: true, champion: true, ability: { heal: 300, cry: 240, cryR: 3600, cryPct: 20 }, housing: 0 },
    rc:          { hp: 2508, speed: 60, range: 1800, dmg: 490, atk: 36, hitAt: 18, ranged: 'spear', pref: 'def', hero: true, ability: { shield: 1250, n: 4, heal: 1000 }, housing: 0 },
    // siege machines
    wallwrecker: { hp: 5500, speed: 30, range: 600, dmg: 325, atk: 39, hitAt: 20, wallMult: 10, pref: 'th', siege: true, housing: 1 },
    flameflinger:{ hp: 1700, speed: 15, range: 6600, dmg: 225, atk: 50, hitAt: 25, ranged: 'fire', splash: 600, pref: 'def', decay: 17, siege: true, housing: 1 },
  };
  // spells (Clash level 1). r in mu, every in ticks; housing counts against spell space, not army space
  const SPELLS = {
    lightning:  { housing: 1, r: 1200, dmg: 150 },                                  // buildings and defending troops
    heal:       { housing: 2, r: 3000, pulses: 41, every: 9, heal: 15 },             // own ground troops (heroes half)
    rage:       { housing: 2, r: 3000, pulses: 60, every: 9, boost: 30, dmgPct: 230, speedAdd: 50 },
    poison:     { housing: 1, r: 2400, pulses: 16, every: 30, dmg: 90 },             // defending troops only
    earthquake: { housing: 1, r: 2100, permille: 145, wallMult: 4 },                 // share of each building's max hp
  };
  // traps (hidden until they go off). trigger / r in mu
  // a trap fills one hex (about 3x3 Clash tiles): stepping onto its hex always sets it off
  const HEX_IN = 1000;
  const TRAPS = {
    bomb:             { trigger: 900,  r: 1800, ground: true },
    giant_bomb:       { trigger: 1200, r: 1800, ground: true },
    spring_trap:      { trigger: 600,  spring: true },
    air_bomb:         { trigger: 2400, r: 1800, air: true },
    seeking_air_mine: { trigger: 2400, single: true, air: true },
    skeleton_trap:    { trigger: 3000, skeletons: true },
  };
  // LEVELS-BEGIN (generated by economy/gen_levels.py from the wiki; x10 integers, index = level)
  const LEVELS = {"barbarian":{"hp":[null,450,540,650,850,1050,1250,1600,2050,2300,2500,2700,2900,3100],"dmg":[null,90,120,150,180,230,260,300,340,380,420,450,480,510]},"archer":{"hp":[null,220,260,290,330,400,440,480,520,560,600,640,680,720,760],"dmg":[null,80,100,130,160,200,220,250,280,310,340,370,400,430,460]},"giant":{"hp":[null,4000,5000,6000,7000,9000,11000,13000,15000,18500,20000,22000,24000,27000,30000],"dmg":[null,240,300,400,480,620,860,1100,1240,1400,1560,1720,1880,2080,2280]},"goblin":{"hp":[null,250,300,360,500,650,800,1050,1260,1460,1660],"dmg":[null,110,140,190,240,320,420,520,620,720,820]},"wallbreaker":{"hp":[null,200,240,290,350,530,720,820,920,1120,1300,1400,1500,1600,1700],"dmg":[null,100,200,250,300,430,550,660,750,860,940,1020,1100,1180,1260]},"wizard":{"hp":[null,750,900,1080,1350,1650,1800,1950,2100,2300,2500,2700,2900,3100,3300],"dmg":[null,750,1050,1350,1875,2550,2775,3000,3225,3450,3675,3900,4125,4350,4650]},"healer":{"hp":[null,5000,7000,9000,12000,15000,16000,17000,18000,19000,20000,21000],"heal":[null,252,336,420,462,504,504,504,532,560,560,574]},"pekka":{"hp":[null,30000,35000,40000,45000,50000,55000,59000,63000,67000,72000,77000,82000,88000],"dmg":[null,4680,5220,5760,6480,7380,8460,9720,10980,12240,13500,14580,15660,16920]},"miner":{"hp":[null,5500,6100,6700,7300,8000,9000,10000,11500,13500,15500,17500,20500],"dmg":[null,1360,1496,1632,1768,1904,2040,2176,2312,2448,2720,2975,3315]},"yeti":{"hp":[null,29000,32000,35000,37000,39000,41000,43000,46500],"dmg":[null,2300,2500,2700,2900,3100,3300,3500,3800]},"etitan":{"hp":[null,72000,77000,82000,87000,92000],"dmg":[null,2700,3000,3300,3600,4050]},"rootrider":{"hp":[null,62000,63500,65000,67000],"dmg":[null,2090,2310,2530,2750]},"thrower":{"hp":[null,22000,23500,25000,26500],"dmg":[null,4750,5250,5500,5750]},"meteorgolem":{"hp":[null,145000,160000,175000],"dmg":[null,4800,5300,5800]},"hogrider":{"hp":[null,2700,3120,3700,4300,5000,5900,7000,8100,8900,9700,10800,12300,13800,15000,17500],"dmg":[null,600,700,800,920,1050,1180,1400,1550,1650,1760,1870,2000,2130,2250,2500]},"valkyrie":{"hp":[null,7500,8500,9500,10500,13000,15000,16500,18000,20000,24000,26000,29000],"dmg":[null,1692,1908,2142,2394,2664,3006,3330,3528,3744,4014,4284,4590]},"golem":{"hp":[null,51000,54000,57000,60000,63000,66000,70000,75000,79000,82000,85000,88000,92000,96000,106000],"dmg":[null,840,960,1080,1200,1320,1440,1560,1680,1800,1920,2040,2160,2280,2400,2880]},"witch":{"hp":[null,3000,3200,4000,4700,5200,5400,5600,6500],"dmg":[null,700,770,980,1155,1295,1400,1540,1820]},"bowler":{"hp":[null,3250,3750,4200,4700,5050,5300,5650,6000,7000,8600],"dmg":[null,1320,1584,1848,2112,2244,2376,2508,2772,3080,3520]},"icegolem":{"dmg":[null,480,560,640,720,800,880,960,1040,1120]},"headhunter":{"hp":[null,3600,4000,4400,5000],"dmg":[null,630,690,750,810]},"appwarden":{"hp":[null,15000,16500,18000,19500],"dmg":[null,1530,1665,1800,1935]},"druid":{"hp":[null,13000,14000,15000,16000,17000,18500],"heal":[null,800,850,900,950,1050,1150]},"ruinwitch":{"hp":[null,23000,25500,28000,30500],"dmg":[null,500,520,540,560]},"balloon":{"hp":[null,1500,1800,2160,2800,3900,5450,6900,8400,9400,10400,11400,12400,13600],"dmg":[null,750,960,1440,2160,3240,4860,5940,7080,7680,8280,8700,9120,9780]},"dragon":{"hp":[null,19000,21000,23000,27000,31000,34000,39000,42000,45000,49000,53000,57000,60000],"dmg":[null,1750,2000,2250,2625,3000,3375,3875,4125,4375,4625,4875,5125,5375]},"babydragon":{"hp":[null,12000,13000,14000,15000,16000,17000,18000,19000,20000,21000,22000,23500],"dmg":[null,750,850,950,1050,1150,1250,1350,1450,1550,1650,1750,1850]},"minion":{"hp":[null,580,630,680,730,780,840,900,960,1020,1080,1140,1200,1300,1400],"dmg":[null,380,410,440,470,500,540,580,620,660,700,740,780,840,920]},"bk":{"hp":[null,14450,14810,15180,15560,15950,16350,16750,17170,17600,18050,18500,18960,19430,19920,20420,20930,21450,21980,22530,23090,23670,24270,24870,25490,26130,26780,27460,28140,28850,29560,30300,31070,31840,32640,33460,34290,35150,36020,36920,37850,38790,39760,40760,41780,42820,43890,44990,46110,47270,48450,49670,50920,52190,53500,54840,56220,57630,59080,60550,62080,63630,65220,66850,68530,70240,72000,73780,75570,77350,79050,80750,82450,84150,85850,87550,89170,90780,92400,94010,95630,96900,98180,99450,100730,102000,103280,104550,105830,107100,108380,109650,110930,112200,113480,114750,116000,117250,118500,119750,121000,122250,123500,124750,126000,127250,128500,129750,131000,132250,133500],"dmg":[null,1224,1248,1260,1296,1320,1344,1380,1392,1428,1464,1488,1524,1548,1584,1608,1644,1668,1716,1740,1776,1812,1848,1884,1932,1968,2004,2040,2076,2124,2172,2208,2256,2304,2352,2400,2436,2484,2544,2592,2640,2808,2868,2928,2988,3048,3108,3180,3240,3312,3384,3456,3528,3600,3684,3768,3840,3924,4008,4092,4188,4260,4344,4440,4524,4620,4716,4800,4896,5004,5100,5208,5304,5412,5508,5616,5700,5796,5880,5976,6072,6156,6228,6312,6396,6480,6564,6636,6720,6804,6888,6972,7044,7128,7212,7296,7380,7464,7548,7632,7716,7800,7884,7968,8052,8136,8220,8304,8388,8472,8556]},"aq":{"hp":[null,5800,5920,6040,6170,6300,6430,6570,6700,6850,6990,7140,7290,7440,7590,7750,7920,8080,8260,8420,8610,8780,8970,9160,9350,9540,9740,9950,10160,10380,10590,10820,11040,11270,11510,11750,12000,12260,12510,12780,13040,13310,13590,13880,14170,14470,14780,15080,15400,15720,16060,16460,16880,17300,17740,18190,18650,19120,19600,20100,20600,21110,21640,22180,22740,23300,23840,24320,24760,25160,25520,25840,26160,26480,26800,27120,27400,27680,27960,28240,28520,28800,29040,29280,29520,29760,30000,30240,30480,30720,30960,31200,31440,31680,31920,32160,32400,32640,32880,33120,33360,33600,33840,34080,34320,34560,34800,35040,35280,35520,35760],"dmg":[null,1020,1042,1072,1095,1125,1155,1178,1215,1238,1268,1298,1335,1372,1402,1440,1470,1508,1552,1590,1628,1672,1710,1755,1800,1845,1890,1935,1980,2032,2085,2138,2190,2242,2302,2362,2415,2482,2535,2602,2670,2738,2805,2872,2948,3022,3098,3172,3255,3338,3420,3488,3555,3638,3712,3788,3862,3945,4028,4110,4192,4275,4358,4448,4538,4628,4710,4785,4860,4920,4980,5032,5078,5115,5152,5190,5228,5258,5295,5325,5355,5378,5408,5430,5460,5482,5505,5535,5558,5588,5610,5632,5662,5685,5715,5738,5760,5782,5805,5828,5850,5872,5895,5918,5940,5962,5985,6008,6030,6052,6075]},"gw":{"hp":[null,8500,8680,8860,9040,9230,9420,9610,9820,10030,10250,10480,10720,10970,11220,11480,11730,11990,12240,12500,12750,13010,13270,13540,13810,14090,14380,14670,14970,15270,15580,15900,16220,16550,16880,17220,17570,17930,18290,18670,19040,19210,19380,19550,19720,19890,20060,20230,20400,20570,20740,20910,21080,21250,21420,21590,21760,21930,22100,22270,22440,22610,22780,22950,23120,23290,23460,23630,23800,23970,24140,24310,24480,24650,24820,24990,25160,25330,25500,25670,25840,26010,26180,26350,26520,26690],"dmg":[null,774,792,828,864,882,918,972,1008,1062,1098,1152,1188,1260,1314,1386,1440,1494,1566,1620,1692,1764,1836,1908,1998,2088,2178,2268,2358,2466,2574,2682,2790,2916,3024,3150,3294,3420,3564,3726,3870,3978,4068,4140,4212,4266,4338,4392,4446,4518,4572,4644,4698,4752,4824,4878,4932,4968,5022,5058,5112,5148,5202,5256,5292,5346,5382,5436,5472,5526,5562,5616,5670,5724,5778,5832,5886,5940,5994,6048,6102,6156,6210,6264,6318,6372]},"rc":{"hp":[null,25080,25500,25930,26350,26780,27200,27630,28050,28480,28900,29330,29750,30180,30600,31030,31450,31880,32300,32730,33150,33490,33830,34170,34510,34850,35190,35530,35870,36210,36550,36810,37060,37320,37570,37830,38080,38340,38590,38850,39100,39360,39610,39870,40120,40380,40640,40900,41160,41420,41680,41940,42200,42460,42720,42980],"dmg":[null,4080,4200,4320,4440,4500,4560,4620,4680,4752,4824,4896,4968,5040,5112,5184,5256,5328,5376,5424,5472,5520,5580,5640,5688,5724,5760,5796,5832,5868,5904,5940,5976,6024,6072,6120,6168,6216,6264,6312,6360,6396,6432,6468,6504,6540,6576,6612,6648,6684,6720,6756,6792,6828,6864,6900]},"lightning":{"dmg":[null,1500,1800,2100,2400,2700,3200,4000,4800,5600,6000,6400,6800,7200]},"heal":{"heal":[null,150,200,250,300,350,400,450,500,550,600,670,750]},"rage":{"pct":[null,2300,2400,2500,2600,2700,2800,2900]},"poison":{"dmg":[null,900,1150,1450,1800,2200,2600,2800,3000,3200,3400,3600,3800]},"earthquake":{"pct":[null,145,170,210,250,290,290,290,290]}};
  // LEVELS-END
  // hero equipment: two pieces per hero set its ability and add stats; without equipment a hero keeps its built-in ability.
  // a value is a number or [base, per step, levels per step]; hpPct / dmgPct in %, dmgUpPct in % while raged or hidden,
  // durations in ticks, heal in hp, quake / qpct = radius (mu) and permille of each structure's hp (walls x3)
  const EQUIP = {
    war_horn:        { hero: 'bk', dmgPct: [2, 1], ab: { rage: [180, 8], dmgUpPct: [25, 3], heal: [150, 40] } },
    barbarian_totem: { hero: 'bk', hpPct: [3, 1], ab: { spawn: 'barbarian', n: [3, 1, 2], heal: [100, 30] } },
    quake_boots:     { hero: 'bk', hpPct: [2, 1], dmgPct: [1, 1, 2], ab: { quake: 2400, qpct: [80, 6], heal: [100, 25] } },
    iron_fist:       { hero: 'bk', epic: true, hpPct: [5, 1], dmgPct: [3, 1], ab: { rage: [240, 10], dmgUpPct: [30, 3], heal: [300, 60], quake: 1800, qpct: [50, 4] } },
    shadow_vial:     { hero: 'aq', dmgPct: [2, 1], ab: { hide: [90, 4], dmgUpPct: [40, 4], heal: [80, 20] } },
    archer_totem:    { hero: 'aq', hpPct: [3, 1], ab: { spawn: 'archer', n: [4, 1, 2], heal: [80, 20] } },
    healer_totem:    { hero: 'aq', hpPct: [2, 1], ab: { spawn: 'healer', n: [1, 1, 4], heal: [100, 25] } },
    hunter_bow:      { hero: 'aq', epic: true, dmgPct: [4, 1], ab: { hide: [120, 4], dmgUpPct: [60, 5], heal: [150, 40] } },
    ancient_tome:    { hero: 'gw', hpPct: [3, 1], ab: { invuln: [60, 4], r: 4200 } },
    life_stone:      { hero: 'gw', hpPct: [4, 1], ab: { allyHeal: [300, 60], r: 4200 } },
    fury_stone:      { hero: 'gw', dmgPct: [2, 1], ab: { allyRage: [150, 8], r: 3600 } },
    ricochet_shield: { hero: 'rc', dmgPct: [2, 1], ab: { shield: [800, 80], n: [3, 1, 4] } },
    royal_jewel:     { hero: 'rc', hpPct: [4, 1], ab: { heal: [800, 120] } },
    // the champion's gear (two worn): cryAdd = Rally Cry +% damage, cryArea = +% reach, heal = on the cry, allyHeal = troops around
    knight_blade:    { hero: 'champion', dmgPct: [3, 1], ab: { cryAdd: [2, 1, 3] } },
    tower_shield:    { hero: 'champion', hpPct: [4, 1], ab: { heal: [150, 40] } },
    commander_helm:  { hero: 'champion', hpPct: [2, 1], ab: { cryArea: [15, 2], cryAdd: [1, 1, 4] } },
    dragon_banner:   { hero: 'champion', epic: true, hpPct: [3, 1, 2], dmgPct: [2, 1, 2], ab: { cryAdd: [4, 1, 2], allyHeal: [200, 40], r: 3600 } },
    // the Champion's Armoury (champ_gear.sql): bought and upgraded with tokens. guard = % less damage taken, leech = % of damage dealt healed
    war_axe:         { hero: 'champion', dmgPct: [4, 1], leech: [2, 1, 3], ab: {} },
    runed_greatsword:{ hero: 'champion', dmgPct: [7, 1], ab: {} },
    flame_brand:     { hero: 'champion', dmgPct: [5, 1], ab: { cryAdd: [2, 1, 3] } },
    stormbreaker:    { hero: 'champion', dmgPct: [9, 1], leech: [3, 1, 4], ab: {} },
    dawnbringer:     { hero: 'champion', dmgPct: [11, 1], leech: [3, 1, 5], ab: { cryAdd: [3, 1, 3] } },
    iron_buckler:       { hero: 'champion', hpPct: [5, 1], guard: [2, 1, 4], ab: {} },
    valor_shield:  { hero: 'champion', hpPct: [7, 1], guard: [3, 1, 4], ab: {} },
    dragonscale_shield:{ hero: 'champion', hpPct: [6, 1], ab: { heal: [200, 40] } },
    bulwark_aegis:   { hero: 'champion', hpPct: [9, 1], guard: [4, 1, 4], ab: {} },
    titan_bulwark:{ hero: 'champion', hpPct: [11, 1], guard: [5, 1, 4], ab: { heal: [300, 50] } },
    bearhide_cap:     { hero: 'champion', hpPct: [3, 1], dmgPct: [2, 1, 2], ab: {} },
    iron_helm:       { hero: 'champion', hpPct: [4, 1], ab: { cryArea: [10, 1] } },
    horned_warhelm:  { hero: 'champion', hpPct: [3, 1, 2], dmgPct: [4, 1], ab: {} },
    crown_of_ash:    { hero: 'champion', hpPct: [4, 1, 2], dmgPct: [4, 1, 2], ab: { cryArea: [20, 2] } },
    lionheart_crown: { hero: 'champion', hpPct: [5, 1, 2], dmgPct: [5, 1, 2], ab: { cryArea: [25, 2], cryAdd: [2, 1, 3] } },
    lucky_coin:      { hero: 'champion', ab: { cryAdd: [2, 1, 3] } },
    healing_amulet:  { hero: 'champion', ab: { allyHeal: [120, 20], r: 3000 } },
    war_drum:        { hero: 'champion', ab: { cryAdd: [3, 1, 2], cryArea: [10, 2] } },
    phoenix_feather: { hero: 'champion', ab: { heal: [250, 40], allyHeal: [100, 20], r: 3000 } },
    heart_of_realm:  { hero: 'champion', dmgPct: [3, 1, 3], ab: { cryAdd: [5, 1, 2], allyHeal: [250, 40], r: 3600 } },
    swift_vial:      { hero: 'rc', dmgPct: [2, 1], ab: { haste: [180, 10], speedAdd: [20, 1], heal: [300, 50] } },
  };
  const eqv = (v, L) => typeof v === 'number' ? v : typeof v === 'string' ? v : v[0] + v[1] * Math.floor((L - 1) / (v[2] || 1));
  // a hero's equipment as the battle uses it: { hpPct, dmgPct, parts: [ability numbers per piece] } (null = built-in ability)
  function heroEquip(hero, list) {
    if (!Array.isArray(list) || !list.length) return null;
    const eq = { hpPct: 0, dmgPct: 0, guardPct: 0, leechPct: 0, parts: [] };
    for (const [k, l] of list.slice(0, hero === 'champion' ? 4 : 2)) {   // the Champion wears four pieces, heroes two
      const E = EQUIP[k]; if (!E || E.hero !== hero) continue;
      const L = Math.max(1, Math.min(30, l | 0)), ab = {};
      if (E.hpPct) eq.hpPct += eqv(E.hpPct, L); if (E.dmgPct) eq.dmgPct += eqv(E.dmgPct, L);
      if (E.guard) eq.guardPct += eqv(E.guard, L); if (E.leech) eq.leechPct += eqv(E.leech, L);
      for (const f of Object.keys(E.ab)) ab[f] = eqv(E.ab[f], L);
      eq.parts.push(ab);
    }
    return eq.parts.length ? eq : null;
  }
  // champion stats: +hpPerLv / +dmgPerLv percent per level above 1, then +2% per Valor rank (exact integer math)
  const CHAMP = { maxLv: 40, maxRank: 20, hpPerLv: 15, dmgPerLv: 19,   // weaker at level 1, steeper growth (owner's decisions 2026-10-24, and 2026-10-26: base 480 / 36 so the $1 Champion does not dominate Keep 1-2; higher levels within 5% of before)
                rankPct: 2, trigger: 4200,
    // stances: chosen from level 10; levels 10 / 20 / 30 are tiers 1 / 2 / 3 (index into each list)
    stances: {
      guardian:  { selfGuard: [20, 25, 30], troopGuard: [10, 15, 20] },            // takes less damage; Rally Cry also shields the troops it reaches
      warlord:   { cryArea: [140, 160, 180], cryTime: [140, 160, 180], cryAdd: [5, 8, 12] },   // a wider, longer, stronger Rally Cry
      berserker: { dmgPct: [10, 15, 20], leech: [10, 12, 15] },                   // hits harder and heals by part of the damage it deals (halved 2026-10-26: it was the only stance worth picking)
    } };
  const champTier = L => (L | 0) >= 30 ? 3 : (L | 0) >= 20 ? 2 : (L | 0) >= 10 ? 1 : 0;
  // a stance number for this champion unit (0 when it has none, or is below level 10)
  function stanceOf(u, k) { const S = u.stance && CHAMP.stances[u.stance], t = champTier(u.lv); return S && t && S[k] ? S[k][t - 1] : 0; }
  function champStat(base, perLv, L, R) {
    L = Math.max(1, Math.min(CHAMP.maxLv, L | 0)); R = Math.max(0, Math.min(CHAMP.maxRank, R | 0));
    return tdiv(tdiv(base * (100 + perLv * (L - 1)), 100) * (100 + CHAMP.rankPct * R), 100);
  }
  // a stat at level L: the level-1 number scaled by the wiki table (exact integer math)
  function lvl(type, field, L, v) {
    const t = LEVELS[type] && LEVELS[type][field];
    return !L || L <= 1 || !t || !t[L] || !t[1] ? v : tdiv(v * t[L], t[1]);
  }
  const RES_TYPES = { gold_mine: 1, elixir_collector: 1, gold_storage: 1, elixir_storage: 1 };
  // defenses: dmg is per shot (dps x interval), range/min in mu, every in ticks
  const DEFS = {
    tower_A:        { range: 6000, min: 0,    dmg: 10, every: 15,  shot: 'arrow',   z: 2000 },
    tower_catapult: { range: 6600, min: 2400, dmg: 25, every: 150, shot: 'boulder', z: 1900, splash: 900 },
  };
  const BHP = { castle: 1600, tower_A: 460, tower_catapult: 450 }, BHP_DEFAULT = 340, WALL_HP = 400;
  const FOOT = { castle: 1250 }, FOOT_DEFAULT = 800;
  const LOOT_ROLE = { castle: 'th', mine: 'gold_mine', windmill: 'elixir_coll', market: 'gold_sto', tavern: 'elixir_sto' };
  const isRes = e => e.kind === 'building' && (RES_TYPES[e.type] === 1 || (LOOT_ROLE[e.id] && LOOT_ROLE[e.id] !== 'th'));

  // sprite for a wall piece from the two directions it connects to (degrees, multiples of 60)
  function wallPiece(e1, e2, gate) {
    const diff = mod(e2 - e1, 360); let piece, rot;
    if (diff === 180) { piece = gate ? 'wall_straight_gate' : 'wall_straight'; rot = mod(e1 - 180, 360); }
    else {
      const sharp = diff === 60 || diff === 300, other = sharp ? 120 : 60;
      for (const [a, b] of [[e1, e2], [e2, e1]]) if (mod(b - (a - 180 + other), 360) === 0) rot = mod(a - 180, 360);
      piece = sharp ? 'wall_corner_B_outside' : 'wall_corner_A_outside';
    }
    return { id: `${piece}_r${rot}`, brokenId: (piece === 'wall_straight_gate' ? 'wall_straight' : piece) + `_r${rot}_broken`, straight: diff === 180 };
  }
  // ---------- base layout (pure function of its parameters) ----------
  function makeBase(opts = {}) {
    const G = opts.G || 8, RW = opts.RW || 5, GATE = RW + (RW >> 1);
    const ents = [];
    const add = (e) => { e.idx = ents.length; ents.push(e); };
    const building = (id, x, y) => {
      const h = BHP[id] || BHP_DEFAULT;
      add({ kind: 'building', id, x, y, hp: h, max: h, foot: FOOT[id] || FOOT_DEFAULT, def: DEFS[id] ? { ...DEFS[id] } : null, loot: null });
    };
    const loop = ring(RW);
    const eang = (a, b) => 60 * DIRS.findIndex(d => d[0] === b[0] - a[0] && d[1] === b[1] - a[1]);
    loop.forEach((t, i) => {
      const e1 = eang(t, loop[mod(i - 1, loop.length)]), e2 = eang(t, loop[(i + 1) % loop.length]), w = wallPiece(e1, e2, i === GATE);
      const [x, y] = hx(...t);
      add({ kind: 'wall', id: w.id, brokenId: w.brokenId, x, y, hp: WALL_HP, max: WALL_HP, arms: [e1, e2] });
      if (!w.straight && (i / RW) % 2 === 0) building('tower_A', x, y + 10);
    });
    building('castle', 0, 0);
    ['barracks', 'church', 'blacksmith', 'market', 'tower_catapult', 'well'].forEach((id, k) => building(id, ...hx(...ring(2)[k * 2])));
    [[3, 'tower_A'], [9, 'tower_catapult'], [15, 'tower_A']].forEach(([k, id]) => building(id, ...hx(...ring(3)[k])));
    ['windmill', 'archeryrange', 'mine', 'home_A', 'tavern', 'home_B', 'lumbermill', 'home_A', 'home_B', 'barracks', 'home_A', 'tavern']
      .forEach((id, k) => building(id, ...hx(...ring(4)[k * 2 + 1])));
    // resources sitting in the base: storages give 20%, collectors 50%, Town Hall its share (Clash rules)
    const res = opts.resources || { gold: 60000, elixir: 60000 };
    const stoG = ents.filter(e => LOOT_ROLE[e.id] === 'gold_sto'), stoE = ents.filter(e => LOOT_ROLE[e.id] === 'elixir_sto');
    for (const e of ents) {
      const role = LOOT_ROLE[e.id]; if (!role) continue;
      if (role === 'th') e.loot = { gold: tdiv(res.gold * 20, 100 * 5), elixir: tdiv(res.elixir * 20, 100 * 5) };
      if (role === 'gold_sto') e.loot = { gold: tdiv(res.gold * 20 * 4, 100 * 5 * stoG.length), elixir: 0 };
      if (role === 'elixir_sto') e.loot = { gold: 0, elixir: tdiv(res.elixir * 20 * 4, 100 * 5 * stoE.length) };
      if (role === 'gold_mine') e.loot = { gold: 2500, elixir: 0 };
      if (role === 'elixir_coll') e.loot = { gold: 0, elixir: 2500 };
    }
    // scenery (renderer only, never touches the battle)
    const scenery = [], rnd = rng(opts.seed || 7);
    const NAT = ['trees_A_large', 'trees_B_large', 'trees_A_medium', 'trees_B_medium', 'trees_A_small'], HILL = ['hills_A_trees', 'hills_B_trees', 'hills_C_trees', 'mountain_A_grass_trees'];
    const [gx, gy] = hx(...loop[GATE]);
    for (let d = RW + 2; d <= G; d++) for (const t of ring(d)) {
      const [x, y] = hx(...t), v = rnd();
      const ex = x - tdiv(gx * d, RW), ey = y - tdiv(gy * d, RW);
      if (ex * ex + ey * ey < 3000 * 3000) continue;
      if (v < 0.05 + 0.05 * (d - RW - 2)) scenery.push({ id: HILL[Math.floor(rnd() * HILL.length)], x, y });
      else if (v < 0.24) scenery.push({ id: NAT[Math.floor(rnd() * NAT.length)], x, y });
    }
    return { version: 1, G, RW, GATE, deployMin: RW + 2, ents, scenery };
  }

  // ---------- navigation ----------
  const CELL = 500;
  function segDist(px, py, ax, ay, bx, by) {
    const dx = bx - ax, dy = by - ay, L2 = dx * dx + dy * dy;
    let t = ((px - ax) * dx + (py - ay) * dy) / L2; t = t < 0 ? 0 : t > 1 ? 1 : t;
    const ex = px - ax - t * dx, ey = py - ay - t * dy;
    return Math.sqrt(ex * ex + ey * ey);
  }
  function wallDist(px, py, e) {
    let m = Infinity;
    for (const a of e.arms) { const v = ARM[a]; m = Math.min(m, segDist(px, py, e.x, e.y, e.x + v[0], e.y + v[1])); }
    return m;
  }
  const edist = (x, y, e) => e.kind === 'wall' ? wallDist(x, y, e) : idist(x, y, e.x, e.y);
  // one nav cell: 0 free, 1 building, 2 wall, 3 off the map; navRef = what blocks it
  function navCell(B, k) {
    const N = B.N, nav = B.nav, ref = B.navRef;
    nav[k] = 0; ref[k] = -1;
    const x = (k % N) * CELL + CELL / 2 - B.EXT, y = Math.floor(k / N) * CELL + CELL / 2 - B.EXT;
    const [q, r] = toHex(x, y);
    if (hdist(q, r) > B.base.G) { nav[k] = 3; return; }
    for (const e of B.ents) {
      if (e.hp <= 0) continue;
      if (e.kind === 'building') { const dx = x - e.x, dy = y - e.y; if (dx * dx + dy * dy < e.foot * e.foot) { nav[k] = 1; ref[k] = e.idx; break; } }
      else { const dx = x - e.x, dy = y - e.y; if (dx * dx + dy * dy < 1200 * 1200 && wallDist(x, y, e) < 420) { nav[k] = 2; ref[k] = e.idx; } }
    }
  }
  function buildNav(B) {
    for (let k = 0; k < B.N * B.N; k++) navCell(B, k);
    B.navVer++;
  }
  // something fell: only the cells it blocked can change (exactly what a full rebuild would give, at a fraction of the cost)
  function dropNav(B, e) {
    const ref = B.navRef;
    for (let k = 0; k < ref.length; k++) if (ref[k] === e.idx) navCell(B, k);
    B.navVer++;
  }
  function cellOf(B, x, y) { const i = Math.floor((x + B.EXT) / CELL), j = Math.floor((y + B.EXT) / CELL); return (i < 0 || j < 0 || i >= B.N || j >= B.N) ? -1 : j * B.N + i; }
  const cellPos = (B, k) => [(k % B.N) * CELL + CELL / 2 - B.EXT, Math.floor(k / B.N) * CELL + CELL / 2 - B.EXT];
  // Which open cells A* can reach from which: one label per connected area, rebuilt only when the map has changed.
  // A diagonal step that A* allows (no building or, without jump, wall on either side cell) always has an open side cell,
  // unless both side cells are off the map; so orthogonal steps plus those all-off-map diagonals connect exactly the same
  // cells for a troop that jumps and one that does not, and one labelling serves both.
  function navComp(B) {
    if (B.compVer === B.navVer) return B.comp;
    const N = B.N, NN = N * N, nav = B.nav;
    if (!B.comp) { B.comp = new Int32Array(NN); B.compStack = new Int32Array(NN); }
    const comp = B.comp, stack = B.compStack;
    comp.fill(-1);
    let lab = 0;
    for (let s = 0; s < NN; s++) {
      if (comp[s] >= 0 || nav[s] === 1 || nav[s] === 3) continue;
      comp[s] = lab; let sp = 0; stack[sp++] = s;
      while (sp) {
        const k = stack[--sp], i = k % N, j = (k - i) / N;
        for (let dj = -1; dj <= 1; dj++) for (let di = -1; di <= 1; di++) {
          if (!di && !dj) continue;
          const ni = i + di, nj = j + dj; if (ni < 0 || nj < 0 || ni >= N || nj >= N) continue;
          const nk = nj * N + ni, t = nav[nk];
          if (t === 1 || t === 3 || comp[nk] >= 0) continue;
          if (di && dj && (nav[j * N + ni] !== 3 || nav[nj * N + i] !== 3)) continue;
          comp[nk] = lab; stack[sp++] = nk;
        }
      }
      lab++;
    }
    B.compVer = B.navVer;
    return comp;
  }
  // A* over the nav grid. Same search, same order, same result as before (2026-10-26 speed-up for the server verifier):
  // the open list pops the least (f, push order), which is a strict order, so any correct heap pops the same cells.
  // box = [x, y, R]: every cell centre the goal accepts lies within R of (x, y) on both axes. When no such cell is in the
  // start's connected area, the search could only end empty, so it returns null without searching (it used to search
  // the whole area first, the bulk of a big battle's cost).
  function astar(B, sx, sy, goalOk, h, jump, box) {
    const N = B.N, NN = N * N, nav = B.nav, EXT = B.EXT, start = cellOf(B, sx, sy); if (start < 0) return null;
    if (box && nav[start] !== 1 && nav[start] !== 3) {
      const comp = navComp(B), c = comp[start], R = box[2];
      const i0 = Math.max(0, Math.floor((box[0] - R + EXT) / CELL) - 1), i1 = Math.min(N - 1, Math.floor((box[0] + R + EXT) / CELL) + 1);
      const j0 = Math.max(0, Math.floor((box[1] - R + EXT) / CELL) - 1), j1 = Math.min(N - 1, Math.floor((box[1] + R + EXT) / CELL) + 1);
      let any = false;
      for (let j = j0; j <= j1 && !any; j++) for (let i = i0; i <= i1; i++) {
        if (comp[j * N + i] === c && goalOk(i * CELL + CELL / 2 - EXT, j * CELL + CELL / 2 - EXT)) { any = true; break; }
      }
      if (!any) return null;
    }
    if (!B.asG || B.asG.length !== NN) { B.asG = new Int32Array(NN); B.asCame = new Int32Array(NN); B.asClosed = new Uint8Array(NN); B.asK = new Int32Array(8 * 20000 + 16); }
    const g = B.asG.fill(0x3fffffff), came = B.asCame.fill(-1), closed = B.asClosed.fill(0), kOf = B.asK;
    // heap of numbers: key = f * 2^21 + push order (push order < 2^18, f < 2^31, so the key is exact and unique)
    const heap = []; let seq = 0;
    const push = (f, k) => {
      kOf[seq] = k; const key = f * 2097152 + seq++;
      let i = heap.length; heap.push(key);
      while (i > 0) { const p = (i - 1) >> 1, pk = heap[p]; if (pk <= key) break; heap[i] = pk; i = p; }
      heap[i] = key;
    };
    const pop = () => {
      const top = heap[0], last = heap.pop(), n = heap.length;
      if (n) {
        let i = 0;
        for (;;) {
          const l = 2 * i + 1; if (l >= n) break;
          const r = l + 1, m = r < n && heap[r] < heap[l] ? r : l;
          if (heap[m] >= last) break;
          heap[i] = heap[m]; i = m;
        }
        heap[i] = last;
      }
      return kOf[top % 2097152];
    };
    g[start] = 0; push(h((start % N) * CELL + CELL / 2 - EXT, Math.floor(start / N) * CELL + CELL / 2 - EXT), start);
    let guard = 0;
    while (heap.length && guard++ < 20000) {
      const k = pop(); if (closed[k]) continue; closed[k] = 1;
      const i = k % N, j = Math.floor(k / N), x = i * CELL + CELL / 2 - EXT, y = j * CELL + CELL / 2 - EXT;
      if (goalOk(x, y)) { const path = []; for (let c = k; c !== -1; c = came[c]) path.push(c); return path.reverse(); }
      for (let dj = -1; dj <= 1; dj++) for (let di = -1; di <= 1; di++) {
        if (!di && !dj) continue;
        const ni = i + di, nj = j + dj; if (ni < 0 || nj < 0 || ni >= N || nj >= N) continue;
        const nk = nj * N + ni, t = nav[nk];
        if (t === 1 || t === 3) continue;
        // no cutting a corner between two blocked cells: buildings always, walls too unless it jumps them (2026-10-24: troops
        // slipped through the gaps where the wall cells of a slanted wall only touch at their corners)
        const sideA = nav[j * N + ni], sideB = nav[nj * N + i];
        if (di && dj && (sideA === 1 || sideB === 1 || (!jump && (sideA === 2 || sideB === 2)))) continue;
        const ng = g[k] + (di && dj ? 14 : 10) + (t === 2 && !jump ? 140 : 0);
        if (ng < g[nk]) { g[nk] = ng; came[nk] = k; push(ng + h(ni * CELL + CELL / 2 - EXT, nj * CELL + CELL / 2 - EXT), nk); }
      }
    }
    return null;
  }
  function clearLine(B, ax, ay, bx, by, jump) {
    const L = idist(ax, ay, bx, by), n = Math.max(1, tdiv(L + 199, 200)), N = B.N;
    const blocked = k => k < 0 || (B.nav[k] && !(jump && B.nav[k] === 2));
    let pk = cellOf(B, ax, ay);
    for (let s = 1; s <= n; s++) {
      const k = cellOf(B, ax + tdiv((bx - ax) * s, n), ay + tdiv((by - ay) * s, n)); if (blocked(k)) return false;
      // a diagonal hop between samples passes the corner of the two cells beside it: both of those must be open too
      if (pk >= 0 && k !== pk && k % N !== pk % N && Math.floor(k / N) !== Math.floor(pk / N)
          && (blocked(Math.floor(pk / N) * N + k % N) || blocked(Math.floor(k / N) * N + pk % N))) return false;
      pk = k;
    }
    return true;
  }

  // ---------- battle ----------
  // opts.cc: the attacker's Clan Castle troops ({type: count}), deployed all at once with the 'cc' command
  function create(base, army, levels, opts) {
    const EXT = 2 * base.G * 1000 + 2000, N = Math.ceil(2 * EXT / CELL);
    const B = {
      base, tick: 0, EXT, N, nav: new Uint8Array(N * N), navRef: new Int32Array(N * N), navVer: 0,
      ents: base.ents.map(e => ({ ...e, arms: e.arms ? e.arms.slice() : undefined, def: e.def ? { ...e.def } : null, loot: e.loot ? { ...e.loot } : null, taken: { gold: 0, elixir: 0, dark: 0 } })),
      army: { ...army }, levels: { ...(levels || {}) }, cc: { ...((opts && opts.cc) || {}) }, equip: (opts && opts.equip) || {}, champ: (opts && opts.champ) || null, units: [], shots: [], fx: [], log: [], loot: { gold: 0, elixir: 0, dark: 0 }, over: false, result: null,
      pathFix: !!(opts && opts.pathFix),   // rules of 2026-10-27 (stepUnit): set for battles started after sg_cfg sim.path_fix_from, so older replays stay exact
    };
    B.ents.forEach((e, i) => { if (e.def) e.cd = (i * 7) % e.def.every; if (e.kind === 'trap') e.armed = !e.spent; });   // spent: set off in an earlier Siege Weekend attack
    B.spells = [];
    B.totalHp = B.ents.filter(e => e.kind === 'building').reduce((s, e) => s + e.max, 0);
    buildNav(B);
    return B;
  }
  function canDeploy(B, type, x, y) {
    if (B.over) return 'battle over';
    if (SPELLS[type]) {
      if (!(B.army[type] > 0)) return 'none left';
      if (!Number.isInteger(x) || !Number.isInteger(y)) return 'bad position';
      return Math.abs(x) > B.EXT || Math.abs(y) > B.EXT ? 'outside the map' : null;
    }
    if (type !== 'cc' && (!TROOPS[type] || TROOPS[type].summoned)) return 'unknown troop';
    if (!(B.army[type] > 0)) return 'none left';
    if (!Number.isInteger(x) || !Number.isInteger(y)) return 'bad position';
    const [q, r] = toHex(x, y), d = hdist(q, r);
    if (d < B.base.deployMin || d > B.base.G) return 'outside deploy zone';
    return null;
  }
  // queue a deploy for the current tick; returns an error string or null
  function deploy(B, type, x, y) {
    const err = canDeploy(B, type, x, y); if (err) return err;
    B.army[type]--;
    B.log.push({ t: B.tick, type, x, y });
    if (!SPELLS[type]) B.housing = (B.housing | 0) + (type === 'cc' ? Object.keys(B.cc).reduce((s, k) => s + (TROOPS[k] ? TROOPS[k].housing * (B.cc[k] | 0) : 0), 0) : TROOPS[type].hero ? 25 : TROOPS[type].housing);
    if (SPELLS[type]) B.spells.push({ type, x, y, t0: B.tick, n: 0, lv: (B.levels[type] | 0) || 1 });
    else if (type === 'cc') { for (const k of Object.keys(B.cc).sort()) for (const u of spawnAround(B, k, B.cc[k] | 0, x, y, -1)) { u.lv = (B.levels[k] | 0) || 1; u.hp = u.max = lvl(k, 'hp', u.lv, TROOPS[k].hp); u.fromCC = true; } }
    else spawn(B, type, x, y).army = true;            // one of the player's own (not summoned, not from the Garrison): its death is counted
    return null;
  }
  function spawn(B, type, x, y, owner = -1) {
    const T = TROOPS[type], L = owner === -1 && !T.summoned ? (B.levels[type] | 0) || 1 : 1, eq = T.hero && owner === -1 ? heroEquip(type, B.equip[type]) : null;
    let hp = lvl(type, 'hp', L, T.hp); if (eq && !T.champion) hp += tdiv(hp * eq.hpPct, 100);
    let cL = 0, cR = 0;
    let cS = null;
    if (T.champion) { const c = B.champ || {}; cL = Math.max(1, c.lv | 0); cR = c.rank | 0; cS = CHAMP.stances[c.stance] ? c.stance : null; hp = champStat(T.hp, CHAMP.hpPerLv, cL, cR); if (eq) hp += tdiv(hp * eq.hpPct, 100); }
    const u = { id: B.units.length, type, x, y, hp, max: hp, lv: L, act: 'run', actTick: B.tick, fx: 0, fy: 1,
                   target: -1, wall: -1, path: [], navVer: -1, fired: false, dead: false, deathTick: 0, owner, born: B.tick, mites: 0, until: {} };
    if (eq) u.eq = eq;
    if (T.champion) { u.lv = cL; u.rank = cR; u.stance = cS; }
    B.units.push(u); return u;
  }
  // n summons on a small ring around (x, y): fixed offsets, no randomness
  const RING8 = [[1, 0], [0, 1], [-1, 0], [0, -1], [1, 1], [-1, 1], [-1, -1], [1, -1]];
  function spawnAround(B, type, n, x, y, owner) { const out = []; for (let i = 0; i < n; i++) { const o = RING8[i % 8], k = 300 + 150 * (i >> 3); out.push(spawn(B, type, x + o[0] * k, y + o[1] * k, owner)); } return out; }
  const on = (B, u, k) => (u.until[k] || 0) > B.tick;
  // defenses can't see burrowing miners or a cloaked queen
  const hidden = (B, u) => (TROOPS[u.type].burrow && u.act === 'run') || on(B, u, 'hide');
  // hero ability (log command 'ability:<hero>'); returns an error string or null
  function useAbility(B, hero) {
    const u = B.units.find(o => o.type === hero && TROOPS[o.type].hero);
    if (!u) return 'hero not deployed'; if (u.dead) return 'hero is dead'; if (u.abilityUsed) return 'ability used';
    u.abilityUsed = true;
    if (TROOPS[hero].champion) return champCry(B, u);
    if (u.eq) { for (const A of u.eq.parts) equipAbility(B, u, A); B.fx.push({ kind: 'heal', x: u.x, y: u.y, t: B.tick }); return null; }
    const A = TROOPS[hero].ability;
    if (A.heal) u.hp = Math.min(u.max, u.hp + A.heal);
    if (A.rage) u.until.rage = B.tick + A.rage;
    if (A.hide) u.until.hide = B.tick + A.hide;
    if (A.spawn) spawnAround(B, A.spawn, A.n, u.x, u.y, u.id);
    if (A.invuln) for (const o of B.units) if (!o.dead && o.side !== 'def' && idist(o.x, o.y, u.x, u.y) <= A.r) o.until.invuln = B.tick + A.invuln;
    if (A.shield) {                                     // seeking shield: the n nearest defenses, else buildings
      const pool = B.ents.filter(e => alive(e) && e.kind === 'building').sort((a, b) => (!!b.def - !!a.def) || (idist(a.x, a.y, u.x, u.y) - idist(b.x, b.y, u.x, u.y)) || (a.idx - b.idx));
      for (const e of pool.slice(0, A.n)) { hurtEnt(B, e, A.shield); B.fx.push({ kind: 'zap', x: e.x, y: e.y, t: B.tick }); }
    }
    B.fx.push({ kind: 'heal', x: u.x, y: u.y, t: B.tick });
    return null;
  }
  // Rally Cry: every attacker nearby (the champion too) hits harder for a while; stance and gear shape it
  function champCry(B, u) {
    const A = TROOPS.champion.ability, parts = u.eq ? u.eq.parts : [], sum = k => parts.reduce((s, p) => s + (p[k] || 0), 0);
    const area = (stanceOf(u, 'cryArea') || 100) + sum('cryArea'), time = stanceOf(u, 'cryTime') || 100, r = tdiv(A.cryR * area, 100), dur = tdiv(A.cry * time, 100), tg = stanceOf(u, 'troopGuard');
    for (const o of B.units) if (!o.dead && o.side !== 'def' && idist(o.x, o.y, u.x, u.y) <= r) {
      o.until.cry = B.tick + dur; o.cryPct = A.cryPct + (u.rank | 0) + stanceOf(u, 'cryAdd') + sum('cryAdd');
      if (tg) { o.until.guard = B.tick + dur; o.guardPct = tg; }
    }
    B.fx.push({ kind: 'cry', x: u.x, y: u.y, r, t: B.tick });
    u.hp = Math.min(u.max, u.hp + A.heal + 25 * (u.rank | 0));
    for (const P of parts) equipAbility(B, u, { heal: P.heal, allyHeal: P.allyHeal, r: P.r });
    B.fx.push({ kind: 'heal', x: u.x, y: u.y, t: B.tick });
    return null;
  }
  // one equipment piece's ability
  function equipAbility(B, u, A) {
    if (A.heal) u.hp = Math.min(u.max, u.hp + A.heal);
    if (A.rage) u.until.rage = Math.max(u.until.rage || 0, B.tick + A.rage);
    if (A.hide) u.until.hide = Math.max(u.until.hide || 0, B.tick + A.hide);
    if (A.spawn && A.n > 0) spawnAround(B, A.spawn, A.n, u.x, u.y, u.id);
    if (A.invuln) for (const o of B.units) if (!o.dead && o.side !== 'def' && idist(o.x, o.y, u.x, u.y) <= A.r) o.until.invuln = B.tick + A.invuln;
    if (A.allyHeal) for (const o of B.units) if (!o.dead && o.side !== 'def' && idist(o.x, o.y, u.x, u.y) <= A.r) { o.hp = Math.min(o.max, o.hp + (TROOPS[o.type].hero && o !== u ? tdiv(A.allyHeal, 2) : A.allyHeal)); B.fx.push({ kind: 'heal', x: o.x, y: o.y, t: B.tick }); }
    if (A.allyRage) for (const o of B.units) if (!o.dead && o.side !== 'def' && idist(o.x, o.y, u.x, u.y) <= A.r) o.until.srage = Math.max(o.until.srage || 0, B.tick + A.allyRage);
    if (A.quake) {
      for (const e of B.ents) if (alive(e) && e.kind !== 'trap' && edist(u.x, u.y, e) <= A.quake) hurtEnt(B, e, Math.max(1, tdiv(e.max * A.qpct * (e.kind === 'wall' ? 3 : 1), 1000)));
      B.fx.push({ kind: 'quake', x: u.x, y: u.y, r: A.quake, t: B.tick });
    }
    if (A.shield) {
      const pool = B.ents.filter(e => alive(e) && e.kind === 'building').sort((a, b) => (!!b.def - !!a.def) || (idist(a.x, a.y, u.x, u.y) - idist(b.x, b.y, u.x, u.y)) || (a.idx - b.idx));
      for (const e of pool.slice(0, A.n)) { hurtEnt(B, e, A.shield); B.fx.push({ kind: 'zap', x: e.x, y: e.y, t: B.tick }); }
    }
  }
  // attacker damage after rage and the wall multiplier
  function hitDmg(B, u, T, e) {
    let n;
    if (u.eq && !T.champion) { const b = lvl(u.type, 'dmg', u.lv, T.dmg); n = b + tdiv(b * (u.eq.dmgPct + (on(B, u, 'rage') || on(B, u, 'hide') ? u.eq.parts.reduce((s, A) => s + (A.dmgUpPct || 0), 0) : 0)), 100); }
    else if (T.champion) { n = champStat(T.dmg, CHAMP.dmgPerLv, u.lv, u.rank); const b = stanceOf(u, 'dmgPct') + (u.eq ? u.eq.dmgPct : 0); if (b) n += tdiv(n * b, 100); }
    else n = lvl(u.type, 'dmg', u.lv, T.dmg) + (on(B, u, 'rage') || on(B, u, 'hide') ? T.ability.dmgUp : 0);
    if (on(B, u, 'cry')) n += tdiv(n * (u.cryPct | 0), 100);
    if (T.champion) { const l = stanceOf(u, 'leech') + (u.eq ? u.eq.leechPct | 0 : 0); if (l && !u.dead) u.hp = Math.min(u.max, u.hp + tdiv(n * l, 100)); }   // berserker, leeching gear
    if (on(B, u, 'srage')) n = tdiv(n * (u.ragePct || SPELLS.rage.dmgPct), 100);   // Rage Spell (its level sets the boost)
    if (e && e.kind === 'wall' && T.wallMult) n *= T.wallMult; if (e && T.resMult && isRes(e)) n *= T.resMult; return n;
  }
  const speedOf = (B, u, T) => T.speed + (on(B, u, 'srage') ? SPELLS.rage.speedAdd : 0) + (u.hasteAdd && on(B, u, 'haste') ? u.hasteAdd : 0);
  const alive = e => e.hp > 0;
  function reachOf(T, e) { return T.range + (e.kind === 'building' ? e.foot : 450); }
  function chooseTarget(B, u, skip = -1) {
    const T = TROOPS[u.type];
    const near = f => { let best = -1, bd = Infinity; for (const e of B.ents) if (alive(e) && e.idx !== skip && f(e)) { const d = idist(e.x, e.y, u.x, u.y); if (d < bd) { bd = d; best = e.idx; } } return best; };
    if (T.pref === 'wall') { const w = near(e => e.kind === 'wall'); return w >= 0 ? w : near(e => e.kind === 'building'); }
    if (T.pref === 'def') { const d = near(e => e.kind === 'building' && e.def); return d >= 0 ? d : near(e => e.kind === 'building'); }
    if (T.pref === 'res') { const r = near(isRes); return r >= 0 ? r : near(e => e.kind === 'building'); }
    if (T.pref === 'th') { const t = near(e => e.id === 'castle'); return t >= 0 ? t : near(e => e.kind === 'building'); }
    return near(e => e.kind === 'building');
  }
  function plan(B, u) {
    u.navVer = B.navVer; u.path = []; u.wall = -1;
    if (u.target < 0) return;
    const tg = B.ents[u.target], T = TROOPS[u.type], reach = reachOf(T, tg);
    const path = astar(B, u.x, u.y, (x, y) => edist(x, y, tg) <= reach,
                       (x, y) => Math.max(0, tdiv((idist(x, y, tg.x, tg.y) - reach - 1000) * 10, CELL)), T.jump,
                       [tg.x, tg.y, reach + (tg.kind === 'wall' ? 1000 : 0) + 1]);
    if (!path) return;
    const wk = T.jump ? -1 : path.findIndex(k => B.nav[k] === 2);
    if (wk >= 0) { u.wall = B.navRef[path[wk]]; path.length = wk; }
    u.path = path.map(k => cellPos(B, k));
  }
  function setAct(B, u, a) { if (u.act !== a) { u.act = a; u.actTick = B.tick; } }
  function hurtEnt(B, e, n) {
    if (!alive(e)) return;
    const before = e.hp; e.hp = Math.max(0, e.hp - n);
    if (e.loot) for (const r of ['gold', 'elixir', 'dark']) { // loot drips out with damage, like Clash
      if (!e.loot[r]) continue;
      const due = tdiv(e.loot[r] * (e.max - e.hp), e.max), got = due - e.taken[r];
      if (got > 0) { e.taken[r] += got; B.loot[r] += got; }
    }
    if (before > 0 && e.hp === 0) {
      B.fx.push({ kind: 'dust', x: e.x, y: e.y, t: B.tick });
      if (e.def && e.def.death) {                      // Bomb Tower blows up on the troops around it
        for (const u of B.units) if (!u.dead && u.side !== 'def' && !TROOPS[u.type].air && !hidden(B, u) && idist(u.x, u.y, e.x, e.y) <= 1650) hurtUnit(B, u, e.def.death);
        B.fx.push({ kind: 'boom', x: e.x, y: e.y, t: B.tick });
      }
      dropNav(B, e);
    }
  }
  function hurtUnit(B, u, n) {
    if (u.dead || on(B, u, 'invuln')) return;
    if (!TROOPS[u.type].air && B.units.some(w => !w.dead && TROOPS[w.type].aura && idist(w.x, w.y, u.x, u.y) <= TROOPS[w.type].aura)) n -= tdiv(n, 6);   // life aura = +20% hp
    if (on(B, u, 'guard')) n -= tdiv(n * (u.guardPct | 0), 100);                 // shielded by a guardian's Rally Cry
    if (u.stance === 'guardian') n -= tdiv(n * stanceOf(u, 'selfGuard'), 100);
    if (u.eq && u.eq.guardPct) n -= tdiv(n * u.eq.guardPct, 100);                // armour that takes the edge off
    u.hp = Math.max(0, u.hp - n);
    const T = TROOPS[u.type];
    if (T.mites) {                                     // a yeti sheds a yetimite for every 1/8 of its hp lost
      const due = Math.min(T.mites.n, tdiv((u.max - u.hp) * T.mites.n, u.max));
      if (due > u.mites) { spawnAround(B, T.mites.type, due - u.mites, u.x, u.y, u.id); u.mites = due; }
    }
    if (u.hp === 0) { u.dead = true; u.deathTick = B.tick; setAct(B, u, 'death'); if (T.death) onDeath(B, u, T.death); }
  }
  function onDeath(B, u, D) {
    if (D.dmg) for (const e of B.ents) if (alive(e) && edist(u.x, u.y, e) <= D.r + (e.kind === 'building' ? e.foot : 0)) hurtEnt(B, e, D.dmg);
    if (D.freeze) for (const e of B.ents) if (e.def && alive(e) && idist(e.x, e.y, u.x, u.y) <= D.r) e.frozen = B.tick + D.freeze;
    if (D.spawn) spawnAround(B, D.spawn, D.n, u.x, u.y, u.id);
    B.fx.push({ kind: D.freeze ? 'freeze' : 'boom', x: u.x, y: u.y, t: B.tick });
  }

  // healer: flies straight to the most hurt ground troop and heals everyone around it
  function stepHealer(B, u, T) {
    let tg = null, worst = 2;
    for (const o of B.units) {
      if (o.dead || o === u || o.side === 'def' || TROOPS[o.type].air || TROOPS[o.type].heal || TROOPS[o.type].siege) continue;
      const f = o.hp / o.max; if (f < worst) { worst = f; tg = o; }
    }
    if (!tg) { setAct(B, u, 'idle'); return; }
    const d = idist(u.x, u.y, tg.x, tg.y);
    if (d > T.range) {
      setAct(B, u, 'run'); const s = Math.min(d, speedOf(B, u, T));
      const nx = u.x + tdiv((tg.x - u.x) * s, d), ny = u.y + tdiv((tg.y - u.y) * s, d), k = cellOf(B, nx, ny);
      if (T.air || (k >= 0 && !B.nav[k])) { u.x = nx; u.y = ny; }
      u.fx = tg.x - u.x; u.fy = tg.y - u.y; return;
    }
    if (T.healN) {
      u.fx = tg.x - u.x; u.fy = tg.y - u.y;
      if (u.act !== 'attack') setAct(B, u, 'attack');
      if ((B.tick - u.actTick) % T.atk === 0) {
        const hurt = B.units.filter(o => !o.dead && o !== u && o.side !== 'def' && !TROOPS[o.type].air && o.hp < o.max && idist(o.x, o.y, u.x, u.y) <= T.range)
          .sort((a, b) => (a.hp * b.max - b.hp * a.max) || (a.id - b.id)).slice(0, T.healN);
        const hv = lvl(u.type, 'heal', u.lv, T.heal);
        for (const o of hurt) { o.hp = Math.min(o.max, o.hp + hv); B.fx.push({ kind: 'heal', x: o.x, y: o.y, t: B.tick }); }
      }
      return;
    }
    u.fx = tg.x - u.x; u.fy = tg.y - u.y;
    if (u.act !== 'attack') { setAct(B, u, 'attack'); }
    if ((B.tick - u.actTick) % T.atk === 0) {
      const hv = lvl(u.type, 'heal', u.lv, T.heal);
      for (const o of B.units) if (!o.dead && o.side !== 'def' && !TROOPS[o.type].air && idist(o.x, o.y, tg.x, tg.y) <= T.healSplash) o.hp = Math.min(o.max, o.hp + hv);
      B.fx.push({ kind: 'heal', x: tg.x, y: tg.y, t: B.tick });
    }
  }
  // defending troops (Skeleton Trap): walk at the nearest attacker on the ground and hit it
  // a troop it could not reach is passed over for 3s (skipG), so it turns to the next one instead of running at it for ever
  const skipped = (B, u, o) => o.id === u.skipG && B.tick < u.skipUntil;
  function stepGuard(B, u, T) {
    let tg = null, bd = Infinity;
    for (const o of B.units) if (!o.dead && o.side !== 'def' && canHit(u.type, o) && !hidden(B, o) && !skipped(B, u, o)) { const d = idist(o.x, o.y, u.x, u.y); if (d < bd) { bd = d; tg = o; } }
    if (!tg) { setAct(B, u, 'idle'); return; }
    meleeUnit(B, u, T, tg, bd);
  }
  // slide directions when a straight charge is blocked: head on, then rotate the heading ±40° and ±75° (cos,sin ×1000,
  // integer so it stays deterministic) to hug around a wall or building instead of running on the spot for ever.
  const MELEE_ROT = [[1000, 0], [766, 643], [766, -643], [259, 966], [259, -966]];
  // one step of a troop chasing another troop. Ground troops go straight while the way is open and take an A* route around
  // buildings when it is not (defenders step over walls; attackers go round them, or jump them if they can). Held off by the crowd or a building for a second while
  // already close: it strikes from where it stands, like a troop crowded out at a building. Far and no closer for
  // three seconds: it gives up on that troop (returns false) and the caller picks another.
  function meleeUnit(B, u, T, tg, d) {
    u.fx = tg.x - u.x; u.fy = tg.y - u.y;
    const over = u.side === 'def' || !!T.jump;   // defenders step over walls (Clash); attackers only when they jump them (2026-10-24)
    const reach = T.range + 300;
    if (u.mTg !== tg.id || d < u.mBest - 5) { u.mTg = tg.id; u.mBest = d; u.mStuck = 0; } else u.mStuck++;
    const crowded = u.mStuck >= TPS && d <= reach + 900;
    if (d > reach && !crowded) {
      if (u.mStuck >= 3 * TPS) { u.skipG = tg.id; u.skipUntil = B.tick + 3 * TPS; u.mTg = -1; u.gPath = null; return false; }
      setAct(B, u, 'run'); const s = Math.min(d, speedOf(B, u, T));
      let wx = tg.x, wy = tg.y;
      const here = cellOf(B, u.x, u.y), inside = here >= 0 && B.nav[here] === 1;   // spawned on its building (Clan Castle): walk straight out
      if (!T.air && !inside) {
        const ex = u.x + tdiv((tg.x - u.x) * (d - reach), d), ey = u.y + tdiv((tg.y - u.y) * (d - reach), d);
        if (clearLine(B, u.x, u.y, ex, ey, over)) u.gPath = null;
        else {
          if (!u.gPath || u.gVer !== B.navVer || B.tick - u.gT >= TPS / 2) {
            const path = astar(B, u.x, u.y, (x, y) => idist(x, y, tg.x, tg.y) <= reach,
                               (x, y) => Math.max(0, tdiv((idist(x, y, tg.x, tg.y) - reach) * 10, CELL)), over, [tg.x, tg.y, reach + 1]);
            u.gPath = path ? path.map(k => cellPos(B, k)) : null; u.gVer = B.navVer; u.gT = B.tick;
          }
          const P = u.gPath;
          if (P) {
            while (P.length > 1 && clearLine(B, u.x, u.y, P[1][0], P[1][1], over)) P.shift();
            while (P.length && idist(P[0][0], P[0][1], u.x, u.y) < 80) P.shift();
            if (P.length) { wx = P[0][0]; wy = P[0][1]; }
          }
        }
      }
      const L = idist(wx, wy, u.x, u.y) || 1, st = Math.min(L, s);
      const dx0 = tdiv((wx - u.x) * 1000, L), dy0 = tdiv((wy - u.y) * 1000, L);   // heading toward the next point, ×1000
      if (T.air || inside) { u.x += tdiv(dx0 * st, 1000); u.y += tdiv(dy0 * st, 1000); return true; }
      for (const [c, si] of MELEE_ROT) {
        const rx = tdiv(dx0 * c - dy0 * si, 1000), ry = tdiv(dx0 * si + dy0 * c, 1000);   // rotate the heading
        const nx = u.x + tdiv(rx * st, 1000), ny = u.y + tdiv(ry * st, 1000), k = cellOf(B, nx, ny);
        if (k >= 0 && B.nav[k] !== 1 && B.nav[k] !== 3 && (over || B.nav[k] !== 2)) { u.x = nx; u.y = ny; return true; }
      }
      return true;   // boxed in this tick; mStuck keeps counting, so it strikes or gives up soon
    }
    if (u.act !== 'attack') { setAct(B, u, 'attack'); u.fired = false; }
    let el = B.tick - u.actTick; const atk = T.atk || (T.draw + T.release), hitAt = T.hitAt || T.draw;
    if (el >= atk) { u.actTick += atk; el -= atk; u.fired = false; }
    if (!u.fired && el >= hitAt) {
      u.fired = true; const n = T.dmg !== undefined ? hitDmg(B, u, T, null) : 0;
      if (T.ranged) B.shots.push({ kind: T.ranged, side: 'atk', x0: u.x, y0: u.y, z0: T.air ? 1500 : 700, ent: -1, unit: tg.id, t0: B.tick, dur: 8 + tdiv(d * 27, 10000), dmg: n });
      else hurtUnit(B, tg, n);
    }
    return true;
  }
  // an attacker that prefers no particular target fights defending troops that come close (Clash)
  // who can strike a flying troop, as in Clash: ranged fighters and flyers, never melee ground troops (a Barbarian can't reach
  // a Baby Dragon), and not every ranged one either (a Bowler's boulder rolls on the ground). Defenses have their own air rule.
  const HITS_AIR = new Set(['archer', 'wizard', 'dragon', 'babydragon', 'minion', 'witch', 'headhunter', 'appwarden', 'etitan', 'thrower', 'aq', 'gw']);
  const canHit = (k, o) => !TROOPS[o.type].air || HITS_AIR.has(k);
  function fightGuard(B, u, T) {
    let tg = null, bd = Math.max(1800, (T.range || 0) + 300);               // a ranged troop turns on a defender within its own range
    for (const o of B.defLive) if (!o.dead && canHit(u.type, o) && !skipped(B, u, o)) { const d = idist(o.x, o.y, u.x, u.y); if (d <= bd) { bd = d; tg = o; } }
    if (!tg) return false;
    return meleeUnit(B, u, T, tg, bd);
  }
  function stepUnit(B, u) {
    if (u.dead) return;
    const T = TROOPS[u.type];
    if (T.decay && (B.tick - u.born) % TPS === TPS - 1) { hurtUnit(B, u, T.decay); if (u.dead) return; }
    if (T.zap && (B.tick - u.born) % T.zap.every === 0) {
      for (const e of B.ents) if (alive(e) && edist(u.x, u.y, e) <= T.zap.r + (e.kind === 'building' ? e.foot : 0)) hurtEnt(B, e, T.zap.dmg);
    }
    if (u.side === 'def') return stepGuard(B, u, T);
    if (T.heal) return stepHealer(B, u, T);
    if (T.pref === 'any' && !T.boom && fightGuard(B, u, T)) return;      // flyers fight the defenders too (Clash)
    if (u.target < 0 || !alive(B.ents[u.target])) { u.target = chooseTarget(B, u); u.navVer = -1; if (u.target < 0) { setAct(B, u, 'idle'); return; } }
    if (u.wall >= 0 && !alive(B.ents[u.wall])) u.navVer = -1;
    const wallIn = u.wall >= 0 && alive(B.ents[u.wall]) && edist(u.x, u.y, B.ents[u.wall]) <= reachOf(T, B.ents[u.wall]);
    if (u.navVer !== B.navVer && !wallIn && !T.burrow) plan(B, u);
    const obj = u.wall >= 0 && alive(B.ents[u.wall]) ? B.ents[u.wall] : B.ents[u.target];
    const d = edist(u.x, u.y, obj);
    if (T.burrow && edist(u.x, u.y, B.ents[u.target]) > reachOf(T, B.ents[u.target]) + 50) {   // underground: straight at the target
      const tg = B.ents[u.target], L = idist(tg.x, tg.y, u.x, u.y), s = Math.min(L, speedOf(B, u, T));
      setAct(B, u, 'run'); u.wall = -1; u.x += tdiv((tg.x - u.x) * s, L); u.y += tdiv((tg.y - u.y) * s, L); u.fx = tg.x - u.x; u.fy = tg.y - u.y;
      return;
    }
    // held back by the crowd: the troops already at the target keep pushing this one back (separation below), so it could run
    // on the spot for ever. No closer for a second and near enough: it fights from where it stands. Far and stuck for three
    // seconds: it picks the nearest other building.
    if (u.lastObj !== obj.idx || d < u.bestD - 5) { u.lastObj = obj.idx; u.bestD = d; u.stuckT = 0; } else u.stuckT++;
    const crowded = u.stuckT >= TPS && d <= reachOf(T, obj) + 900;
    if (!crowded && u.stuckT >= 3 * TPS && obj === B.ents[u.target]) {
      const other = chooseTarget(B, u, u.target);
      if (other >= 0) { u.target = other; u.navVer = -1; u.stuckT = 0; u.lastObj = -1; return; }
    }
    if (d > reachOf(T, obj) + 50 && !crowded) {
      setAct(B, u, 'run');
      while (u.path.length > 1 && clearLine(B, u.x, u.y, u.path[1][0], u.path[1][1], T.jump)) u.path.shift();
      let wp = u.path[0] || [obj.x, obj.y], dx = wp[0] - u.x, dy = wp[1] - u.y, L = idist(wp[0], wp[1], u.x, u.y);
      if (L < 80) {
        u.path.shift(); if (u.path.length) return;
        // the last waypoint is a cell centre inside the reach, but the troop can still stand a little outside it:
        // step straight at the target instead of re-planning the same one-cell path every tick (it used to freeze there)
        u.navVer = -1; wp = [obj.x, obj.y]; dx = wp[0] - u.x; dy = wp[1] - u.y; L = idist(wp[0], wp[1], u.x, u.y); if (!L) return;
      }
      const s = Math.min(L, speedOf(B, u, T)), nx = u.x + tdiv(dx * s, L), ny = u.y + tdiv(dy * s, L), nk = cellOf(B, nx, ny);
      // the last steps go straight at the target; a wall on that line is broken, never walked through (2026-10-24: troops
      // walked through a wall that stood between the cell the path ended on and their target)
      // ... but mid-path (pathFix), a step that clips a wall's corner is a nudge from the crowd, not a wall in the way: plan again from
      // here instead of locking onto that wall (live 2026-10-27: troops hit the wall beside the gap they were walking through)
      if (B.pathFix && !T.air && !T.jump && nk >= 0 && B.nav[nk] === 2 && cellOf(B, u.x, u.y) !== nk && B.navRef[nk] !== u.wall && u.path.length) { u.navVer = -1; return; }
      if (!T.air && !T.jump && nk >= 0 && B.nav[nk] === 2 && cellOf(B, u.x, u.y) !== nk && B.navRef[nk] !== u.wall) { u.wall = B.navRef[nk]; u.path = []; u.fx = dx; u.fy = dy; return; }
      u.x = nx; u.y = ny; u.fx = dx; u.fy = dy;
      return;
    }
    u.fx = obj.x - u.x; u.fy = obj.y - u.y;
    if (T.boom) {
      for (const e of B.ents) if (alive(e) && edist(u.x, u.y, e) < T.boomR) hurtEnt(B, e, lvl(u.type, 'dmg', u.lv, e.kind === 'wall' ? T.boom : T.boomOther));
      B.fx.push({ kind: 'boom', x: u.x, y: u.y, t: B.tick }); u.hp = 0; u.dead = true; u.deathTick = B.tick; setAct(B, u, 'death');
      return;
    }
    if (T.ranged === 'arrow') {
      if (u.act !== 'draw' && u.act !== 'release') setAct(B, u, 'draw');
      const el = B.tick - u.actTick;
      if (u.act === 'draw' && el >= T.draw) {
        setAct(B, u, 'release');
        B.shots.push({ kind: 'arrow', side: 'atk', x0: u.x, y0: u.y, z0: 600, ent: obj.idx, unit: -1, t0: B.tick, dur: 6 + tdiv(Math.trunc(d) * 21, 10000), dmg: hitDmg(B, u, T, obj) });
      } else if (u.act === 'release' && el >= T.release) setAct(B, u, 'draw');
      return;
    }
    if (u.act !== 'attack') { setAct(B, u, 'attack'); u.fired = false; }
    let el = B.tick - u.actTick;
    if (el >= T.atk) { u.actTick += T.atk; el -= T.atk; u.fired = false; }
    if (T.summon && (B.tick - u.born) % T.summon.every === T.summon.every - 1) {
      const live = B.units.filter(o => o.owner === u.id && !o.dead).length;
      if (live < T.summon.max) spawnAround(B, T.summon.type, Math.min(T.summon.n, T.summon.max - live), u.x, u.y, u.id);
    }
    if (!u.fired && el >= T.hitAt) {
      u.fired = true;
      if (T.ranged) B.shots.push({ kind: T.ranged, side: 'atk', x0: u.x, y0: u.y, z0: T.air ? 1500 : 700, ent: obj.idx, unit: -1, t0: B.tick, dur: 8 + tdiv(Math.trunc(d) * 27, 10000),
      else if (T.spin) { for (const e of B.ents) if (alive(e) && edist(u.x, u.y, e) <= T.range + T.spin + (e.kind === 'building' ? e.foot : 0)) hurtEnt(B, e, hitDmg(B, u, T, e)); }
      else hurtEnt(B, obj, hitDmg(B, u, T, obj));
    }
  }
  // which troops a defense or trap can hit: 'none' = ground only, 'both', 'only' = air only
  const airOk = (rule, u) => TROOPS[u.type].air ? rule === 'both' || rule === 'only' : rule !== 'only';
  function stepDefense(B, e) {
    const D = e.def;
    if ((e.frozen || 0) > B.tick) return;
    if (D.hidden && !e.revealed) {                     // Hidden Tesla: pops up when a troop comes close
      for (const u of B.units) if (!u.dead && u.side !== 'def' && !hidden(B, u) && idist(u.x, u.y, e.x, e.y) <= D.trigger) { e.revealed = B.tick; break; }
      if (!e.revealed) return;
    }
    if (D.wake && (B.housing | 0) < D.wake) return;     // Eagle: asleep until the attacker has put this much housing on the field
    if (e.cd > 0) { e.cd--; return; }
    let best = null, bd = Infinity;
    const air = D.air || (D.shot === 'arrow' ? 'both' : 'none');
    const ok = u => !u.dead && u.side !== 'def' && !hidden(B, u) && airOk(air, u);
    if (D.ramp && e.lock >= 0 && ok(B.units[e.lock])) {      // Inferno: stays on its target while it can
      const u = B.units[e.lock], d = idist(u.x, u.y, e.x, e.y); if (d <= D.range && d >= D.min) { best = u; bd = d; }
    }
    if (!best) for (const u of B.units) { if (!ok(u)) continue; const d = idist(u.x, u.y, e.x, e.y); if (d <= D.range && d >= D.min && d < bd) { bd = d; best = u; } }
    if (!best) { if (D.ramp) e.lock = -1; return; }
    e.cd = D.every;
    if (D.burst) { e.shotN = (e.shotN | 0) + 1; e.cd = e.shotN % D.burst.n === 0 ? D.burst.rest : D.burst.gap; }   // Eagle: salvos, then a long reload
    if (D.ramp) {                                      // Inferno: the beam heats up the longer it stays on one target
      if (e.lock !== best.id) { e.lock = best.id; e.heat = 0; } else e.heat += D.every;
      let m = 1000; for (const [t, pm] of D.ramp) if (e.heat >= t) m = pm;
      hurtUnit(B, best, Math.max(1, tdiv(D.dmg * m, 1000))); B.fx.push({ kind: 'inferno', x0: e.x, y0: e.y, x: best.x, y: best.y, u: best.id, heat: m, t: B.tick }); return;
    }
    if (D.push) {                                      // Air Sweeper: blows every flyer in a 120 degree cone straight back
      const ax = best.x - e.x, ay = best.y - e.y;
      for (const u of B.units) {
        if (u.dead || !TROOPS[u.type].air) continue;
        const vx = u.x - e.x, vy = u.y - e.y, L = idist(u.x, u.y, e.x, e.y);
        if (L > D.range || 2 * (vx * ax + vy * ay) < L * bd) continue;
        u.x += tdiv(vx * D.push, L || 1); u.y += tdiv(vy * D.push, L || 1);
      }
      B.fx.push({ kind: 'gust', x: best.x, y: best.y, x0: e.x, y0: e.y, t: B.tick }); return;
    }
    if (D.shot === 'tesla') { hurtUnit(B, best, D.dmg); B.fx.push({ kind: 'tesla', x0: e.x, y0: e.y, x: best.x, y: best.y, t: B.tick }); return; }
    if (D.shot !== 'boulder' && D.shot !== 'eagle') B.shots.push({ kind: D.shot, side: 'def', x0: e.x, y0: e.y, z0: D.z, ent: -1, unit: best.id, t0: B.tick, dur: 6 + tdiv(bd * 21, 10000), dmg: D.dmg, splash: D.splash, air: D.splash ? air : undefined });
    else B.shots.push({ kind: D.shot, side: 'def', x0: e.x, y0: e.y, z0: D.z, ent: -1, unit: -1, tx: best.x, ty: best.y, t0: B.tick, dur: 33 + tdiv(bd * 18, 10000), dmg: D.dmg, splash: D.splash, air: D.air === 'both' ? 'both' : undefined });
  }
  function stepCastle(B, e) {
    for (const u of B.units) {
      if (u.dead || u.side === 'def' || hidden(B, u) || idist(u.x, u.y, e.x, e.y) > e.cc.trigger) continue;
      e.ccOut = B.tick;
      for (const k of Object.keys(e.cc.troops).sort()) {
        if (!TROOPS[k]) continue;
        for (const d of spawnAround(B, k, e.cc.troops[k] | 0, e.x, e.y, -1)) {
          d.side = 'def'; d.lv = (e.cc.levels && e.cc.levels[k] | 0) || 1; d.hp = d.max = lvl(k, 'hp', d.lv, TROOPS[k].hp);
        }
      }
      B.fx.push({ kind: 'dust', x: e.x, y: e.y, t: B.tick }); return;
    }
  }
  // the defender's champion steps off its banner when an attacker comes close
  function stepChampion(B, e) {
    for (const u of B.units) {
      if (u.dead || u.side === 'def' || hidden(B, u) || idist(u.x, u.y, e.x, e.y) > (e.champ.trigger || CHAMP.trigger)) continue;
      e.champOut = B.tick;
      const T = TROOPS.champion, d = spawn(B, 'champion', e.x + 300, e.y + 300, -1);
      d.side = 'def'; d.lv = Math.max(1, e.champ.lv | 0); d.rank = e.champ.rank | 0; d.stance = CHAMP.stances[e.champ.stance] ? e.champ.stance : null;
      const eq = heroEquip('champion', e.champ.eq); if (eq) d.eq = eq; else delete d.eq;
      d.hp = d.max = champStat(T.hp, CHAMP.hpPerLv, d.lv, d.rank) + (eq ? tdiv(champStat(T.hp, CHAMP.hpPerLv, d.lv, d.rank) * eq.hpPct, 100) : 0);
      B.fx.push({ kind: 'cry', x: e.x, y: e.y, r: 1800, t: B.tick }); return;
    }
  }
  function stepTrap(B, e) {
    const R = TRAPS[e.type], P = e.trap || {};
    let hit = null;
    for (const u of B.units) {
      if (u.dead || u.side === 'def' || hidden(B, u) || (R.air ? !TROOPS[u.type].air : TROOPS[u.type].air)) continue;
      if (idist(u.x, u.y, e.x, e.y) <= Math.max(R.trigger, HEX_IN)) { hit = u; break; }
    }
    if (!hit) return;
    e.armed = false; e.sprung = B.tick;
    B.fx.push({ kind: R.air ? 'boom' : R.spring ? 'gust' : 'boom', small: !R.r, x: e.x, y: e.y, t: B.tick });
    if (R.single) hurtUnit(B, hit, P.dmg);
    else if (R.r) { for (const u of B.units) if (!u.dead && u.side !== 'def' && !hidden(B, u) && !!TROOPS[u.type].air === !!R.air && idist(u.x, u.y, e.x, e.y) <= R.r + (P.extra || 0)) hurtUnit(B, u, P.dmg); }
    else if (R.spring) {                               // throws the biggest troops that fit its capacity off the field (heroes and siege are too heavy)
      const near = B.units.filter(u => !u.dead && u.side !== 'def' && !TROOPS[u.type].air && !TROOPS[u.type].hero && !TROOPS[u.type].siege && idist(u.x, u.y, e.x, e.y) <= Math.max(R.trigger, HEX_IN))
        .sort((a, b) => (TROOPS[b.type].housing - TROOPS[a.type].housing) || (a.id - b.id));
      let room = P.cap || 10;
      for (const u of near) { const h = Math.max(1, TROOPS[u.type].housing); if (h > room) continue; room -= h; u.hp = 0; u.dead = true; u.deathTick = B.tick; setAct(B, u, 'death'); }
    } else if (R.skeletons) { for (const k of spawnAround(B, 'skeleton', P.n || 2, e.x, e.y, -1)) k.side = 'def'; }
  }
  function stepSpell(B, sp) {
    const S = SPELLS[sp.type], el = B.tick - sp.t0;
    if (sp.done) return;
    const inR = (x, y, r) => idist(x, y, sp.x, sp.y) <= r;
    if (sp.type === 'lightning' || sp.type === 'earthquake') {
      sp.done = true;
      for (const e of B.ents) {
        if (!alive(e) || (e.kind === 'wall' && sp.type === 'lightning') || !inR(e.x, e.y, S.r + (e.kind === 'building' ? e.foot : 0))) continue;
        hurtEnt(B, e, sp.type === 'lightning' ? lvl('lightning', 'dmg', sp.lv, S.dmg) : Math.max(1, tdiv(e.max * lvl('earthquake', 'pct', sp.lv, S.permille) * (e.kind === 'wall' ? S.wallMult : 1), 1000)));
      }
      if (sp.type === 'lightning') for (const u of B.units) if (!u.dead && u.side === 'def' && inR(u.x, u.y, S.r)) hurtUnit(B, u, lvl('lightning', 'dmg', sp.lv, S.dmg));
      B.fx.push({ kind: sp.type === 'lightning' ? 'bolt' : 'quake', x: sp.x, y: sp.y, r: S.r, t: B.tick }); return;
    }
    if (el % S.every !== 0) return;
    if (++sp.n >= S.pulses) sp.done = true;
    for (const u of B.units) {
      if (u.dead || !inR(u.x, u.y, S.r)) continue;
      const own = u.side !== 'def';
      const hv = lvl('heal', 'heal', sp.lv, S.heal);
      if (sp.type === 'heal' && own) u.hp = Math.min(u.max, u.hp + (TROOPS[u.type].hero ? tdiv(hv, 2) : hv));   // flyers too, as in Clash (only the Healer troop can't heal them)
      if (sp.type === 'rage' && own) { u.until.srage = B.tick + S.boost; u.ragePct = lvl('rage', 'pct', sp.lv, S.dmgPct); }
      if (sp.type === 'poison' && !own) hurtUnit(B, u, lvl('poison', 'dmg', sp.lv, S.dmg));
    }
    B.fx.push({ kind: 'spell_' + sp.type, x: sp.x, y: sp.y, r: S.r, t: B.tick });
  }
  function shotTarget(B, s) {
    if (s.unit >= 0) { const u = B.units[s.unit]; return [u.x, u.y]; }
    if (s.ent >= 0) { const e = B.ents[s.ent]; return [e.x, e.y]; }
    return [s.tx, s.ty];
  }
  function step(B) {
    if (B.over) return;
    // the defending troops alive as the troops move (in unit order): defenders only come out after this loop (Clan Castle,
    // champion, Skeleton Trap), so fightGuard can look through these instead of every unit ever spawned
    B.defLive = B.units.filter(u => !u.dead && u.side === 'def');
    for (const u of B.units) stepUnit(B, u);
    for (const e of B.ents) if (e.kind === 'trap' && e.armed) stepTrap(B, e);
    // released once: ccOut / champOut hold the tick they came out, which can be tick 0 (a troop dropped right by the castle)
    for (const e of B.ents) if (e.cc && e.ccOut == null && alive(e)) stepCastle(B, e);
    for (const e of B.ents) if (e.champ && e.champOut == null && alive(e)) stepChampion(B, e);
    for (const sp of B.spells) stepSpell(B, sp);
    for (const e of B.ents) if (e.def && alive(e)) stepDefense(B, e);
    // separation, never into blocked cells
    // only live ground troops ever push (nothing dies here), in unit order; a pair 420 or more apart on either axis is
    // at least 420 apart, which pushes nothing, so it is skipped before the square root. Positions are worked on in
    // plain arrays (same numbers, same order) and written back to the troops that moved.
    const us = B.units.filter(u => !u.dead && !TROOPS[u.type].air), n = us.length;
    const X = new Float64Array(n), Y = new Float64Array(n), moved = new Uint8Array(n), nav = B.nav;
    for (let i = 0; i < n; i++) { X[i] = us[i].x; Y[i] = us[i].y; }
    for (let i = 0; i < n; i++) for (let j = i + 1; j < n; j++) {
      const dx = X[j] - X[i], dy = Y[j] - Y[i];
      if (dx >= 420 || dx <= -420 || dy >= 420 || dy <= -420) continue;
      const d = idist(X[i], Y[i], X[j], Y[j]), min = 420;
      if (d < min) {
        const p = tdiv(min - d, 2), nx = d ? dx : 1, ny = d ? dy : 0, L = d || 1;
        const ax = X[i] - tdiv(nx * p, L), ay = Y[i] - tdiv(ny * p, L), bx = X[j] + tdiv(nx * p, L), by = Y[j] + tdiv(ny * p, L);
        const ka = cellOf(B, ax, ay), kb = cellOf(B, bx, by);
        if (ka >= 0 && !nav[ka]) { X[i] = ax; Y[i] = ay; moved[i] = 1; }
        if (kb >= 0 && !nav[kb]) { X[j] = bx; Y[j] = by; moved[j] = 1; }
      }
    }
    for (let i = 0; i < n; i++) if (moved[i]) { us[i].x = X[i]; us[i].y = Y[i]; }
    for (const s of B.shots) {
      if (s.done || B.tick < s.t0 + s.dur) continue;
      s.done = true;
      const [tx, ty] = shotTarget(B, s);
      if (s.side === 'def') {
        if (s.splash) { for (const u of B.units) if (!u.dead && u.side !== 'def' && airOk(s.air || 'none', u) && !hidden(B, u) && idist(u.x, u.y, tx, ty) < s.splash) hurtUnit(B, u, s.dmg); B.fx.push({ kind: s.kind === 'wiz' ? 'zap' : 'boom', small: true, x: tx, y: ty, t: B.tick }); }
        else hurtUnit(B, B.units[s.unit], s.dmg);
      } else if (s.unit >= 0) hurtUnit(B, B.units[s.unit], s.dmg);   // an attacker's shot at a defending troop
      else if (s.splash) {
        const base = s.wallMult > 1 && B.ents[s.ent].kind === 'wall' ? tdiv(s.dmg, s.wallMult) : s.dmg;
        for (const e of B.ents) if (alive(e) && idist(e.x, e.y, tx, ty) < s.splash + 600) hurtEnt(B, e, e.kind === 'wall' ? base * s.wallMult : base);
        B.fx.push({ kind: s.kind === 'fire' ? 'boom' : 'zap', small: s.kind === 'fire', x: tx, y: ty, t: B.tick });
      } else {
        const tg = B.ents[s.ent]; hurtEnt(B, tg, s.dmg);
        if (s.bounce) {                                // bowler: the ball rolls on to the nearest thing behind
          let nb = null, bd = Infinity;
          for (const e of B.ents) if (e !== tg && alive(e)) { const dd = edist(tx, ty, e); if (dd <= s.bounce && dd < bd) { bd = dd; nb = e; } }
          if (nb) { hurtEnt(B, nb, s.dmg); B.fx.push({ kind: 'dust', x: nb.x, y: nb.y, t: B.tick }); }
        }
      }
    }
    B.shots = B.shots.filter(s => !s.done);
    B.fx = B.fx.filter(f => B.tick - f.t < 24);
    B.tick++;
    const buildingsLeft = B.ents.some(e => e.kind === 'building' && alive(e));
    const armyLeft = Object.keys(B.army).some(k => B.army[k] > 0 && (TROOPS[k] || k === 'cc')) || B.units.some(u => !u.dead && u.side !== 'def') || B.shots.length;
    if (B.tick >= MAX_TICKS || !buildingsLeft || (B.log.length && !armyLeft)) finish(B);
  }
  function destruction(B) {
    const left = B.ents.filter(e => e.kind === 'building').reduce((s, e) => s + e.hp, 0);
    return tdiv((B.totalHp - left) * 100, B.totalHp);
  }
  function finish(B) {
    B.over = true;
    const pct = destruction(B), th = B.ents.find(e => e.id === 'castle');
    const stars = pct === 100 ? 3 : (pct >= 50 ? 1 : 0) + (th && !alive(th) ? 1 : 0);
    B.result = { ticks: B.tick, destruction: pct, stars, loot: { ...B.loot }, hash: hash(B),
                 taken: B.ents.filter(e => e.taken.gold || e.taken.elixir || e.taken.dark).map(e => ({ idx: e.idx, bid: e.bid ?? null, gold: e.taken.gold, elixir: e.taken.elixir, dark: e.taken.dark })),
                 // the player's troops that fell (heroes and the champion never do): only these must be trained again (2026-10-24)
                 dead: B.units.filter(u => u.army && u.dead && !TROOPS[u.type].hero).reduce((o, u) => (o[u.type] = (o[u.type] || 0) + 1, o), {}),
                 // what fell (quests and achievements count these; the server replays the battle, so they can be trusted)
                 wrecked: { walls: B.ents.filter(e => e.kind === 'wall' && !alive(e)).length, buildings: B.ents.filter(e => e.kind === 'building' && !alive(e)).length,
                            defenses: B.ents.filter(e => e.kind === 'building' && e.def && !alive(e)).length, keep: th && !alive(th) ? 1 : 0 },
                 // where every structure stands at the end (Siege Weekend damage carries over to the next attack)
                 hp: B.ents.map(e => e.hp), spent: B.ents.filter(e => e.kind === 'trap' && !e.armed).map(e => e.idx) };
  }
  // player ends the battle early (recorded so the server stops at the same tick)
  function ability(B, hero) {
    if (B.over) return 'battle over';
    if (!TROOPS[hero] || !TROOPS[hero].hero) return 'not a hero';
    const err = useAbility(B, hero); if (err) return err;
    B.log.push({ t: B.tick, type: 'ability:' + hero, x: 0, y: 0 }); return null;
  }
  function endBattle(B) { if (B.over) return; B.log.push({ t: B.tick, type: 'end' }); finish(B); }
  // FNV-1a over everything that matters for the outcome
  function hash(B) {
    let h = 0x811c9dc5;
    const mix = n => { n |= 0; for (let i = 0; i < 4; i++) { h ^= (n >>> (i * 8)) & 0xff; h = Math.imul(h, 0x01000193); } };
    mix(B.tick); mix(B.loot.gold); mix(B.loot.elixir); if (B.loot.dark) mix(B.loot.dark);
    for (const e of B.ents) mix(e.hp);
    for (const u of B.units) { mix(u.x); mix(u.y); mix(u.hp); }
    return (h >>> 0).toString(16).padStart(8, '0');
  }
  // server side: rebuild the battle from the deploy log alone and return the authoritative result
  function replay(base, army, log, levels, opts) {
    const B = create(base, army, levels, opts);
    let i = 0, lastT = -1, work = 0;
    // Deterministic compute tripwire for the server verifier: the per-tick cost grows with the unit array (dead units
    // stay in it), so a summon-heavy battle is the dominant cost. Housing already bounds the worst legal battle to
    // ~50M unit-ticks; this cap sits well above that, so a legal log never trips it, but a pathological log is refused
    // (-> 'bad log' -> a lost raid) instead of tying up the verifier. It counts sim steps, so client and server agree.
    const WORK_CAP = 120000000;
    while (!B.over) {
      while (i < log.length && log[i].t === B.tick) {
        const c = log[i++];
        if (c.t < lastT) return { error: 'log out of order', at: i - 1 };
        lastT = c.t;
        if (c.type === 'end') { endBattle(B); break; }
        const err = typeof c.type === 'string' && c.type.startsWith('ability:') ? ability(B, c.type.slice(8)) : deploy(B, c.type, c.x, c.y);
        if (err) return { error: err, at: i - 1 };
      }
      if (i < log.length && log[i].t < B.tick) return { error: 'log out of order', at: i };
      step(B);
      work += B.units.length;
      if (work > WORK_CAP) return { error: 'bad log' };
    }
    if (i < log.length) return { error: 'deploys after battle end', at: i };
    return B.result;
  }
  return { TPS, MU, MAX_TICKS, TROOPS, SPELLS, TRAPS, DEFS, EQUIP, CHAMP, champStat, champTier, heroEquip, wallPiece, DIRS, makeBase, create, deploy, ability, canDeploy, step, replay, endBattle, hash, destruction, hx, toHex, hdist, ring, idist, lvl };   // lvl: the army cards' stat tips (play.html)
});
