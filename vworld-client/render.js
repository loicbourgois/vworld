const render = () => {
  if (document.getElementById('get_true_ping').checked) {
    return
  }
  const zoom = parseFloat(document.querySelector("#slider_1").value) / 1000.0 * 9.0 + 1.0
  if (!chunk.step) {
    return
  }
  // fps
  last_render_time_ms = now_ms
  now_ms = Date.now()
  const elapsed_ms = now_ms - last_render_time_ms
  fps_list.push(1.0 / (elapsed_ms / 1000.0))
  while (fps_list.length > 10) {
    fps_list.shift()
  }
  let fps_sum = 0;
  for (let i = 0 ; i < fps_list.length ; i += 1) {
    fps_sum += fps_list[i]
  }
  const fps = fps_sum / fps_list.length;
  const simulation_time_s = (chunk.step * chunk.constants.delta_time)
  const real_time_s = (chunk.real_time_ms / 1000)
  const simulation_speed = simulation_time_s / real_time_s
  context_1.clearRect(0, 0, canvas_1.width, canvas_1.height)
  context_2.clearRect(0, 0, canvas_2.width, canvas_2.height)
  for (let particle_id in chunk.particles) {
    const particle = chunk.particles[particle_id]
    if (particle.type_ == "Plant") {
      draw_plant(canvas_1, particle.x, particle.y, particle.diameter, zoom, center_x, center_y, particle.color)
      draw_plant(canvas_2, particle.x, particle.y, particle.diameter, 1.0, 0.5, 0.5, particle.color)
    } else {
      draw_body(canvas_1, particle.x, particle.y, particle.diameter, zoom, center_x, center_y)
      draw_body(canvas_2, particle.x, particle.y, particle.diameter, 1.0, 0.5, 0.5)
    }
  }
  for (let particle_id in chunk.particles) {
    const particle = chunk.particles[particle_id]
    if (particle.type_ == "Eye") {
      const x = particle.x + particle.direction.x * particle.diameter * 0.3;
      const y = particle.y + particle.direction.y * particle.diameter * 0.3;
      draw_eye(canvas_1, x, y, particle.diameter, zoom, center_x, center_y, particle.output)
      draw_body_up(canvas_1, particle.x, particle.y, particle.diameter*0.65, zoom, center_x, center_y)
    } else if (particle.type_ == "Mouth") {
      let x = particle.x + particle.direction.x * particle.diameter * 0.35;
      let y = particle.y + particle.direction.y * particle.diameter * 0.35;
      draw_mouth(canvas_1, x, y, particle.diameter, zoom, center_x, center_y, particle.output)
      draw_body_up(canvas_1, particle.x, particle.y, particle.diameter*0.65, zoom, center_x, center_y)
    } else if (particle.type_ == "Turbo") {
      log_x_time(2, particle);
      const x = particle.x + particle.direction.x * particle.diameter * 0.3;
      const y = particle.y + particle.direction.y * particle.diameter * 0.3;
      draw_turbo(canvas_1, x, y, particle.diameter, zoom, center_x, center_y, particle.output)
      draw_body_up(canvas_1, particle.x, particle.y, particle.diameter*0.65, zoom, center_x, center_y)
    }
  }
  if (document.getElementById('show_outputs').checked) {
    for (let particle_id in chunk.particles) {
      const particle = chunk.particles[particle_id]
      if (particle.type_ == "Plant") {
        // Do nothing
      } else {
        draw_output(canvas_1, particle.x, particle.y, particle.diameter, zoom, center_x, center_y, particle.output)
      }
    }
  }
  if (document.getElementById('show_health').checked) {
    for (let particle_id in chunk.particles) {
      const particle = chunk.particles[particle_id]
      if (particle.type_ == "Plant") {
        // Do nothing
      } else {
        draw_energy(canvas_1, particle.x, particle.y, particle.diameter, zoom, center_x, center_y, particle.energy * conf.health_diameter_ratio)
      }
    }
  }
  document.getElementById('fps').innerHTML = fps.toFixed(2);
  document.getElementById('step').innerHTML = chunk.step;
  document.getElementById('simulation_time').innerHTML = tohhmmss(simulation_time_s);
  document.getElementById('real_time').innerHTML = tohhmmss(real_time_s);
  document.getElementById('simulation_speed').innerHTML = simulation_speed.toFixed(5);
  document.getElementById('simulation_current_speed').innerHTML = chunk.stats[chunk.stats.length-1].simulation_speed.toFixed(5);
  document.getElementById('distance_traveled_as_fitness_function').checked = chunk.constants.use_distance_traveled_as_fitness_function
  const left = canvas_2.width * (center_x  - 0.5 / zoom)
  const top = canvas_2.height * (center_y - 0.5  / zoom)
  const width = canvas_2.width / zoom
  const height = canvas_2.height / zoom
  context_2.strokeStyle = '#fff'
  context_2.beginPath();
  context_2.rect(left, top, width, height);
  context_2.stroke();
  context_3.clearRect(0, 0, canvas_3.width, canvas_3.height)
  context_4.clearRect(0, 0, canvas_4.width, canvas_4.height)
  render_stats_age();
  render_stats_distance();
  return;




  //
  // Old
  //





  document.getElementById('particles_count').innerHTML = chunk.particles_count;
  document.getElementById('entities_count').innerHTML = chunk.entities_count;
  document.getElementById('links_count').innerHTML = chunk.links_count;
  document.getElementById('total_energy').innerHTML = chunk.total_energy;

  context_1.clearRect(0, 0, canvas_1.width, canvas_1.height)
  context_2.clearRect(0, 0, canvas_2.width, canvas_2.height)
  context_3.clearRect(0, 0, canvas_3.width, canvas_3.height)
  context_4.clearRect(0, 0, canvas_4.width, canvas_4.height)

  if (document.getElementById('show_travel_line').checked) {
    for (let euuid in chunk.entities) {
      const entity = chunk.entities[euuid]
      draw_line(entity.x_start, entity.y_start, entity.x, entity.y, zoom, conf.colors.travel)
    }
  }
  if (document.getElementById('show_line_of_sight').checked) {
    for (let particle_id in chunk.particles) {
      const particle = chunk.particles[particle_id]
      if (particle.type_ == "Eye") {
        const x = particle.x + particle.data.EyeData.direction.x * chunk.constants.eye_sight_length;
        const y = particle.y + particle.data.EyeData.direction.y * chunk.constants.eye_sight_length;
        draw_line(particle.x, particle.y, x, y, zoom, conf.colors.line_of_sight)
      }
    }
  }
  for (let particle_id in chunk.particles) {
    const particle = chunk.particles[particle_id]
    if (particle.type_ == "Plant") {
      draw_plant(canvas_1, particle.x, particle.y, particle.diameter, zoom, center_x, center_y, particle.data.PlantData.color)
      draw_plant(canvas_2, particle.x, particle.y, particle.diameter, 1.0, 0.5, 0.5, particle.data.PlantData.color)
    } else {
      draw_body(canvas_1, particle.x, particle.y, particle.diameter, zoom, center_x, center_y)
      draw_body(canvas_2, particle.x, particle.y, particle.diameter, 1.0, 0.5, 0.5)
    }
  }
  for (let particle_id in chunk.particles) {
    const particle = chunk.particles[particle_id]
    if (particle.type_ == "Mouth") {
      try {
        let x = particle.x + particle.data.MouthData.direction.x * particle.diameter * 0.35;
        let y = particle.y + particle.data.MouthData.direction.y * particle.diameter * 0.35;
        if (Math.abs(particle.data.MouthData.direction.x) < 0.1 && Math.abs(particle.data.MouthData.direction.y)  < 0.1)
        {
          x = particle.x + 0.0 * particle.diameter * 0.35;
           y = particle.y - 1.0 * particle.diameter * 0.35;
        }
        draw_mouth(canvas_1, x, y, particle.diameter, zoom, center_x, center_y)
      } catch (error) {

      }
    }
  }
  for (let particle_id in chunk.particles) {
    const particle = chunk.particles[particle_id]
    if (particle.type_ == "Plant") {
      // Do nothing
    } else {
      draw_body(canvas_1, particle.x, particle.y, particle.diameter*0.75, zoom, center_x, center_y)
    }
  }
  for (let particle_id in chunk.particles) {
    const particle = chunk.particles[particle_id]
    if (particle.type_ == "Eye") {
      try {
        const x = particle.x + particle.data.EyeData.direction.x * particle.diameter * 0.3;
        const y = particle.y + particle.data.EyeData.direction.y * particle.diameter * 0.3;
        draw_eye(canvas_1, x, y, particle.diameter, zoom, center_x, center_y)
      } catch (error) {

      }
    }
  }
  for (let particle_id in chunk.particles) {
    const particle = chunk.particles[particle_id]
    if (particle.type_ == "Plant") {
      // Do nothing
    } else {
      draw_body(canvas_1, particle.x, particle.y, particle.diameter*0.65, zoom, center_x, center_y)
    }
  }
  if (document.getElementById('show_outputs').checked) {
    for (let particle_id in chunk.particles) {
      const particle = chunk.particles[particle_id]
      if (particle.type_ == "Plant") {
        // Do nothing
      } else {
        draw_output(canvas_1, particle.x, particle.y, particle.diameter, zoom, center_x, center_y, particle.output)
      }
    }
  }
  if (document.getElementById('show_health').checked) {
    for (let particle_id in chunk.particles) {
      const particle = chunk.particles[particle_id]
      if (particle.type_ == "Plant") {
        // Do nothing
      } else {
        draw_energy(canvas_1, particle.x, particle.y, particle.diameter, zoom, center_x, center_y, particle.energy * 0.8)
      }
    }
  }

  if (document.getElementById('show_health_big').checked) {
    for (let particle_id in chunk.particles) {
      const particle = chunk.particles[particle_id]
      if (particle.type_ == "Plant") {
        // Do nothing
      } else {
        draw_energy(canvas_1, particle.x, particle.y, particle.diameter, zoom, center_x, center_y, particle.energy)
      }
    }
  }
  if (document.getElementById('show_links').checked) {
    for (let link_id in chunk.links) {
      const link = chunk.links[link_id]
      const p1 = chunk.particles[link.puuids_str[0]]
      const p2 = chunk.particles[link.puuids_str[1]]
      draw_link(p1.x, p1.y, p2.x, p2.y, zoom)
    }
  }
}
const render_stats_distance = () => {
  let l = chunk.stats.length;
  let max_distance = chunk.best_dna_ever_by_distance_traveled.distance_traveled;
  max_distance = max_distance ? max_distance : 0.0;
  let last_distance_alive = chunk.stats[l-1].best_dna_alive_by_age.distance_traveled;
  document.getElementById('best_ever_distance_traveled').innerHTML = max_distance.toFixed(5);
  document.getElementById('best_alive_distance_traveled').innerHTML = last_distance_alive.toFixed(5);
  const resolution = parseFloat(document.querySelector("#resolution").value)
  let step = l / resolution;
  for (let i = 0; i < l ; i += step) {
      let stat = chunk.stats[Math.trunc(i)]
      const x = stat.step / chunk.step * canvas_3.width;
      [
        'best_dna_alive_by_age',
        'best_dna_ever_by_age',
        'best_dna_alive_by_distance_traveled',
        'best_dna_ever_by_distance_traveled',
        'averages',
      ].forEach(element => {
        let p = {
          x: x,
          y: (1.0 - stat[element].distance_traveled  / max_distance) * canvas_3.height
        }
        draw_stat_point(canvas_3, p, conf.colors[element])
      });
  }
}
const render_stats_age = () => {
  let l = chunk.stats.length;
  let max_age = chunk.best_dna_ever_by_age.age_in_ticks;
  let max_age_alive = chunk.best_dna_alive_by_age.age_in_ticks;
  document.getElementById('best_ever_age_in_ticks').innerHTML = max_age;
  document.getElementById('best_alive_age_in_ticks').innerHTML = max_age_alive;
  const resolution = parseFloat(document.querySelector("#resolution").value)
  let step = l / resolution;
  for (let i = 0; i < l ; i += step) {
    let stat = chunk.stats[Math.trunc(i)]
    const x = stat.step / chunk.step * canvas_4.width;
    [
      'best_dna_alive_by_distance_traveled',
      'best_dna_ever_by_distance_traveled',
      'best_dna_alive_by_age',
      'best_dna_ever_by_age',
      'averages',
    ].forEach(element => {
      let p = {
        x: x,
        y: (1.0 - stat[element].age_in_ticks / max_age) * canvas_4.height
      }
      draw_stat_point(canvas_4, p, conf.colors[element])
    });
  }
}
const draw_stat_point = (canvas, p, color) => {
  const radius_canvas = 0.01 * 0.5 * canvas.width;
  const startAngle = 0;
  const endAngle = Math.PI + (Math.PI * 360) * 0.5;
  const context = canvas.getContext('2d')
  context.beginPath();
  context.arc(p.x, p.y, radius_canvas, startAngle, endAngle);
  context.fillStyle = color;
  context.fill();
}
const get_canvas_cursor_position = (canvas, event) => {
    const rect = canvas.getBoundingClientRect()
    const x = event.clientX - rect.left
    const y = event.clientY - rect.top
    return {
      x: x,
      y: y
    }
}
const draw_line = (x1, y1, x2, y2, zoom, color) => {
  const p1 = get_canvas_coord(canvas_1, x1, y1, zoom, center_x, center_y)
  const p2 = get_canvas_coord(canvas_1, x2, y2, zoom, center_x, center_y)
  context_1.beginPath()
  context_1.moveTo(p1.x, p1.y)
  context_1.lineTo(p2.x, p2.y)
  context_1.lineWidth = 2;
  context_1.strokeStyle = color
  context_1.stroke()
}
const draw_link = (x1, y1, x2, y2, zoom) => {
  draw_line(x1, y1, x2, y2, zoom, conf.colors.link)
}
const draw_disk = (canvas, x, y, diameter, zoom, center_x, center_y, color) => {
  const p = get_canvas_coord(canvas, x, y, zoom, center_x, center_y)
  const radius_canvas = diameter * 0.5 * canvas.width * zoom;
  const startAngle = 0;
  const endAngle = Math.PI + (Math.PI * 360) * 0.5;
  const context = canvas.getContext('2d')
  context.beginPath();
  //try {
    context.arc(p.x, p.y, radius_canvas, startAngle, endAngle);
  //} catch (e) {
  //  log_x_time(3, [p.x, p.y, radius_canvas, startAngle, endAngle])
  //}
  context.fillStyle = color;
  context.fill();
}
const draw_eye = (canvas, x, y, diameter, zoom, center_x, center_y, particle_output) => {
  let g = 255.0;
  let r = 255.0 - 255.0 *  particle_output * 0.75;
  let b = 255.0 - 255.0 * particle_output * 0.5;
  draw_disk(canvas, x, y, diameter * 0.65, zoom, center_x, center_y, `rgb(${r}, ${g}, ${b})`)
  draw_disk(canvas, x, y, diameter * 0.45, zoom, center_x, center_y, conf.colors.eye.black)
}
const draw_turbo = (canvas, x, y, diameter, zoom, center_x, center_y, particle_output) => {
  let r = conf.colors.turbo.back.r * particle_output;
  let g = conf.colors.turbo.back.g * particle_output;
  let b = conf.colors.turbo.back.b * particle_output;
  draw_disk(canvas, x, y, diameter * 0.7, zoom, center_x, center_y, `rgb(${r}, ${g}, ${b})`)
  r = conf.colors.turbo.top.r * particle_output;
  g = conf.colors.turbo.top.g * particle_output;
  b = conf.colors.turbo.top.b * particle_output;
  draw_disk(canvas, x, y, diameter * 0.55, zoom, center_x, center_y, `rgb(${r}, ${g}, ${b})`)
}
const draw_mouth = (canvas, x, y, diameter, zoom, center_x, center_y, particle_output) => {
  let r = conf.colors.mouth.back.r * (particle_output* 0.75 + 0.5);
  let g = conf.colors.mouth.back.g * (particle_output* 0.75 + 0.5);
  let b = conf.colors.mouth.back.b * (particle_output* 0.75 + 0.5);
  draw_disk(canvas, x, y, diameter * 0.7, zoom, center_x, center_y, `rgb(${r}, ${g}, ${b})`)
  r = conf.colors.mouth.top.r * (particle_output* 0.5 + 0.5);
  g = conf.colors.mouth.top.g * (particle_output* 0.5 + 0.5);
  b = conf.colors.mouth.top.b * (particle_output* 0.5 + 0.5);
  draw_disk(canvas, x, y, diameter * 0.55, zoom, center_x, center_y, `rgb(${r}, ${g}, ${b})`)

}
const draw_body = (canvas, x, y, diameter, zoom, center_x, center_y) => {
  draw_disk(canvas, x, y, diameter, zoom, center_x, center_y, conf.colors.body)
}
const draw_body_up = (canvas, x, y, diameter, zoom, center_x, center_y) => {
  draw_disk(canvas, x, y, diameter, zoom, center_x, center_y, conf.colors.body_up)
}
const draw_plant = (canvas, x, y, diameter, zoom, center_x, center_y, color_rgb) => {
  draw_disk(canvas, x, y, diameter, zoom, center_x, center_y,
    `rgb(${Math.trunc(color_rgb.r*255.0)}, ${Math.trunc(color_rgb.g*255.0)}, ${Math.trunc(color_rgb.b*255.0)})`
  )
}
const draw_energy = (canvas, x, y, diameter, zoom, center_x, center_y, energy) => {
  diameter = Math.max(0.0, diameter * ( energy / chunk.constants.energy_max ))
  draw_disk(canvas, x, y, diameter, zoom, center_x, center_y, conf.colors.health)
}
const draw_output = (canvas, x, y, diameter, zoom, center_x, center_y, output) => {
  const r = 255.0 * output;
  const g = 255.0 * output;
  const b = 0.0;//(1.0 - output) * 255.0;
  // const a = 0.5;
  const color = `rgba(${r}, ${g}, ${b})`
  draw_disk(canvas, x, y, diameter*0.8, zoom, center_x, center_y, color)
}
const get_canvas_coord = (canvas, x, y, zoom, center_x, center_y) => {
  y = 1.0 - y
  x = x * zoom
  x = x - center_x * zoom + 0.5
  y = y * zoom
  y = y - center_y * zoom + 0.5
  return {
    x: canvas.width * x,
    y: canvas.height * y
  }
}
