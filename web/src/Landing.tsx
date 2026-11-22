export default function Landing() {
  return (
    <div className="landing">
      <header className="topbar"><div className="brand">epidemicsim<small>agent-based epidemics, parallel backends</small></div></header>
      <section className="hero">
        <h1>Simulate an epidemic across a whole city, then see how fast each kind of parallelism can do it.</h1>
        <p>One agent-based model (1M people, 4 steps a day) run by sequential, multithreaded, SIMD, Metal, CUDA and MPI backends that all produce identical results.</p>
        <a className="btn primary big" href="#/demo">One-click demo</a>
        <span className="note"> Replays the pre-baked Mumbai combined-interventions run.</span>
      </section>
      <div className="landing-cards">
        <a className="card landing-card" href="#/studio"><h2>Studio</h2><p>Pick a city, build an intervention scenario (masks, vaccination, isolation, closures, lockdown), run it live or replay it, and compare runs on maps and SEIR curves.</p><b>Open the Studio →</b></a>
        <a className="card landing-card" href="#/lab"><h2>PAGP Lab</h2><p>Measured results for ten parallel-computing topics: speedup and Amdahl, ILP, threads and schedules, SIMD, GPUs, warps, PTX, MPI, OpenACC, and a backend x hardware matrix.</p><b>Open the Lab →</b></a>
      </div>
    </div>
  )
}
