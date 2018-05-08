# Future work

## Simulation

### Implementation improvements

- Think of better name for project!
- Experiment with both "struct of arrays" and "array of structs" for modeling boids.
- Use of "margins" in wrap around code is currently a bit of a hack.
- Make simulation frame independent (fix your time step article is great)
- Use sentinel values in spatial grid to allow exactly the number of requested boids.
- Speed up computation with a parallel collection library like Rayon.
- Dynamically select correct shell gap starting size.
- Lose unnecessary use of Box, e.g in neighbourhood lookup.
- Sort the neighbourhood lookup arrays into memory access pattern order.
- Really dig down into runtime perf - use testing tools to find hotspots, bad caching
- Dont recalculate the forces of each boid every frame, just enough of them.
  (expose the amount/strategy for doing this to the user?)
- Partition the boids and spread the update calculation over several frames.

### Ideas

#### Up next

- Allow simulation parameters to be supplied via config file
- Have several presets for different kinds of flock
- Option to support cursor interaction only when pressing down the mouse button.

#### Maybe one day

- Support automatically reloading config file when it changes.
- Dynamically calculate pleasing default parameters based on window size and resolution.
- Further explore the feel of the simulation.
    * Different sized neighbourhood lookup table patterns.
    * Can we detect how busy the neighbourhood is and use it to scale repulsion,
      based only on some immediate/sampled neighbours positions?
      - Could such a "panic factor" overcome MAX_FORCE? Have a dynamic max force?
      - You could infer than an area is busy from extreme closeness
      - Maybe we can take a cue from reynolds subsumption architecture?
        Disable one behaviour in favour of another?
    * Allow user to tradeoff between perf and accuracy.
    * Throw in randomness or bias to partial sorting algorithm
    * When things are busy/crowded/"angry":
        - Use a dynamic neighbourhood range (don't need big range for "calm" flock)
        - Sample neighbourhood
        - Add a random "panic" force

## Renderer

### Implementation improvements

- Handle resizing of screen.
- Use `hidpi_factor` to scale `gl_PointSize`.

### Ideas

#### Up Next

- Offer more than one renderers/shader.
- Pretty colours!

#### Maybe one day

- Render velocity somehow?
- Support switching between different renderers/shaders at run time.
- Allow running at different resolutions.


## Fps Counter

- Rethink how caching works, maybe this doesn't live in `fps` module.
- Consider building an ring buffer of `Instant` instead of `Duration`

## Main / Application

### Implementation improvements

- Dont rely on vsync to keep 60fps
- Update glutin
- Apply cargo fmt
- Slim down size of main loop
- Reconsider how to process glutin events, current implementation has redundant structs.
  (Re-implements a lot of window events)
