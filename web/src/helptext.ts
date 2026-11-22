import type { HelpText } from './components/Help'

const h = (what: string, effect: string, tryIt: string): HelpText => ({ what, effect, try: tryIt })

/** Help copy for the Studio. Each statement mirrors crates/scenarios (spec.rs, engine.rs) and crates/core (schedule.rs, params.rs, backend.rs). */
export const HELP = {
  preset: h(
    'A ready-made set of interventions. "baseline" has none.',
    'Picking one loads its toggles and sliders. Changing any control turns it into a custom scenario.',
    'Replay baseline, then another preset, and compare the cards.'),
  masks: (efficacy: number): HelpText => h(
    'Masks cut the chance of infection for every contact, at home, work and markets.',
    `Transmission is multiplied by 1 - compliance x efficacy (efficacy is ${Math.round(efficacy * 100)}% here, not adjustable).`,
    'Turn masks on, then compare with and without.'),
  maskCompliance: h(
    'The share of people who wear masks.',
    'Higher means a bigger cut in transmission. At 0% masks do nothing.',
    'Set compliance to 0% and compare with 80%.'),
  start: h(
    'The day the intervention switches on (day 0 is the first day).',
    'Later means the epidemic grows unchecked for longer first, so the peak is usually higher.',
    'Compare day 0 with day 30.'),
  vax: h(
    'Gives vaccine doses each day until the coverage target is reached.',
    'Vaccinated people are less likely to be infected (about 70% at full effect, reached after about 14 days, by default) and to need hospital (about 90%). It is not sterilising.',
    'Switch vaccination on and see what the infection peak does.'),
  vaxStrategy: h(
    'Who gets the doses. Random: anyone not yet vaccinated, alive and not in hospital. Age: only people 60+. Location: blocks of 1000 homes with the most current infections first.',
    'The same number of doses saves different numbers of people depending on who gets them.',
    'Run random, then location, at the same coverage.'),
  coverage: h(
    'The most people that will be vaccinated, as a share of the whole population.',
    'Higher means more doses in total. The rate below decides how fast they are given.',
    'Compare 20% with 60%.'),
  rate: h(
    'Doses given per day, as a share of the whole population.',
    'Higher reaches the coverage target sooner. Lower spreads the doses over more days.',
    'Raise the rate and see if the peak falls.'),
  iso: h(
    'People with symptoms stay at home. It acts on symptomatic agents only; infectious people without symptoms still go out.',
    'Higher compliance keeps more symptomatic people away from work and markets. Each person either always complies or never does.',
    'Set compliance to 0% and compare with 80%.'),
  isoCompliance: h(
    'The share of symptomatic people who stay home.',
    'Higher keeps more of them away from work and markets. At 0% isolation does nothing.',
    'Set compliance to 0% and compare with 80%.'),
  collegeClosure: h(
    'Colleges close and their students stay home.',
    'Removes college contacts only. Schools and workplaces stay open.',
    'Use a threshold trigger so closure starts only if infections rise.'),
  marketClosure: h(
    'Neighbourhood markets close and shoppers stay home.',
    'Removes market contacts only. It matters most at weekends, when more people visit markets.',
    'Compare market closure with no closure.'),
  lockdown: h(
    'Workplaces, schools, colleges and markets all close and people stay home. It is on or off, with no strength setting.',
    'Cuts work, school, college and market contacts. Contacts at home remain.',
    'Compare a 30-day lockdown with one that lasts until the end.'),
  trigMode: h(
    'What starts the closures and lockdown above: a fixed day, or a threshold on a metric.',
    'A threshold is checked once a day. When the metric reaches it, the measure switches on and stays on (until its duration ends).',
    'Switch to threshold and compare early and late triggers.'),
  trigMetric: h(
    'The number the threshold watches. Infectious: share of people who are infectious. Hospital: hospital beds in use divided by capacity. Cumulative: share ever infected.',
    'Infectious reacts early, hospital reacts late, cumulative only ever rises.',
    'Compare infectious and hospital with the same lockdown.'),
  trigThreshold: h(
    'The level of the chosen metric at which closures start.',
    'A lower threshold acts earlier, so a smaller peak. A higher one waits longer.',
    'Compare 0.5% with 2% of the population.'),
  closureDay: h(
    'The day closures and lockdown start (used when the trigger is a fixed day).',
    'Earlier lowers the peak. Later lets the epidemic grow first.',
    'Compare day 7 with day 40.'),
  duration: h(
    'How many days the closures and lockdown last after they start. The infinity sign means until the end of the run.',
    'Short closures can end while many are still infected, and cases may rise again afterwards.',
    'Compare 15 days with until-end.'),
  backend: h(
    'The code that runs the simulation: sequential, ilp, rayon, openmp, simd, metal (Apple GPU) or cuda (NVIDIA GPU).',
    'They give identical results for the same settings and differ only in speed. Not all are available on every machine.',
    'Run the same scenario on sequential and rayon and compare the speed.'),
  agents: h(
    'How many people a live run simulates. Capped (live runs use a sample of the city\'s households).',
    'More agents are closer to the full city but slower. Fewer are quick but noisier.',
    'Try 50k, then 250k.'),
  chains: h(
    'Keeps a record of who may have infected whom, for the agent inspector.',
    'Needed to see infection chains. It adds bookkeeping each step. Sources are co-located infectious people, not a single known infector.',
    'Leave on, run live and click a map cell.'),
  runLive: h(
    'Runs the simulation on the server with the settings above and streams it for 120 days.',
    'Works for any custom setting, in about the time of the run. Only live runs support the agent inspector.',
    'Press Run live twice with different settings to compare.'),
  replay: h(
    'Plays a pre-computed run of a preset (needs the preset to be baked for this city).',
    'Instant, and cards and charts use the average of many runs with 5-95% bands. Disabled once you change any control.',
    'Replay baseline, then combo.'),
  exportJson: h(
    'Downloads the current scenario as JSON, in the same shape as the scenario files.',
    'Lets you save or share a custom scenario. It contains settings only, not results.',
    'Tweak some sliders, export, then open the file.'),
  exportPng: h(
    'Saves an image of the current runs: maps, metric cards and charts.',
    'It shows whatever is on screen, including up to 3 runs side by side.',
    'Run two scenarios, then export the comparison.'),
  speed: h(
    'How fast playback moves, in simulated days per second.',
    'Higher plays faster. For live runs it also sets the pace of the engine.',
    'Slow it down around the peak.'),
  time: h(
    'The point in time shown on the maps and counts. Four steps make one day.',
    'Dragging it scrubs through the run. It pauses following a live run until you click Jump to live.',
    'Drag it to the peak day and compare the maps.'),
  guide: h(
    'Shows every explanation at once.',
    'Useful for presenting. Hover the "?" marks when it is off.',
    'Turn it on for a screenshot.'),
  counts: h(
    'People per disease state now. S susceptible (never infected), E exposed (infected, not yet infectious), I infectious (with or without symptoms), R recovered, D dead.',
    'People in hospital are infected too, so S + E + I + R + D can be lower than the population.',
    'Watch S fall and R rise as the epidemic passes.'),
  heatmap: h(
    'A map of where infected people live: exposed, infectious or in hospital, counted per grid cell by home.',
    'A stronger red means more infected people in that cell, relative to the busiest cell so far in this run. It is not a rate.',
    'Pause at the peak, then scrub to the end.'),
  metricsDelta: h(
    'The small line under each value compares that run with baseline in the same city.',
    'Green means lower than baseline (better). Red means higher. For peak day it shows days earlier or later.',
    'Run baseline and a preset to see deltas.'),
  peakInfections: h(
    'The most people infectious at the same time.',
    'Lower is better for hospitals. Interventions flatten this peak.',
    'Compare it with baseline.'),
  peakDay: h(
    'The day on which the number infectious was highest.',
    'Later means the epidemic was slowed down.',
    'Check how far a lockdown pushes it.'),
  attackRate: h(
    'The share of the population infected at least once by the end of the run.',
    'Lower means fewer people ever infected. Vaccinated people can still be infected.',
    'Compare masks with isolation.'),
  deaths: h(
    'Total deaths by the end of the run.',
    'Falls with fewer infections and with fewer people hospitalised. Hospitalisation and death risk rise with age.',
    'Compare age-targeted with random vaccination.'),
  daysOver: h(
    'Days on which more people were in hospital than there are beds.',
    'Higher means overloaded hospitals. Capacity is the city\'s beds scaled to the number of agents.',
    'Find a scenario that keeps this at 0.'),
  hospital: h(
    'People in hospital over time. The dashed red line is bed capacity.',
    'Above the line, hospitals are overloaded.',
    'Compare scenarios and watch who stays under the line.'),
  rt: h(
    'The effective reproduction number: how many people each infectious person infects, as a daily mean.',
    'Above 1 the epidemic grows. Below 1 it declines. Interventions push it down.',
    'Look for when it first drops below the R = 1 line.'),
  band: h(
    'The shaded band is the 5th to 95th percentile across many runs of the same scenario.',
    'A wide band means results vary a lot between runs. Single live runs have no band.',
    'Replay a preset and compare band widths.'),
  hotspots: h(
    'New infections per day, stacked by the zone of the city where they happened. The five worst zones are shown.',
    'Shows where to focus. The line below says the share in homes, work/school, markets and hospitals.',
    'Compare zones with and without a lockdown.'),
  inspector: h(
    'Details of one person: age, state, daily schedule and who may have infected them.',
    'Click a map cell in a live run to inspect someone who lives there (an infected person if there is one).',
    'Run live with chains on and click a red cell.'),
} as const
