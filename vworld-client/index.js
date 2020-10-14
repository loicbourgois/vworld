'use strict';
const conf = {
  'colors': {
    'health': '#8fa',
    'body':   '#8cf',
    'travel': '#a88',
    'link':   '#eee',
    'best_dna_ever_by_distance_traveled': '#faa',
    'best_dna_ever_by_age': '#afa',
    'best_dna_alive_by_age': '#aaf',
    'best_dna_alive_by_distance_traveled': '#ffa',
    'eye': {
      'white': '#eee',
      'black': '#111'
    },
    'mouth': {
      'black': '#800'
    },
    'line_of_sight': '#aaa'
  },
  'urls': [
    'ws://127.0.0.1:10001',
    'ws://vworld9khrgwnf-vworld.functions.fnc.fr-par.scw.cloud',
  ]
}
const data = {
  socket_pairs: []
}
for (let i = 0 ; i < conf.urls.length ; i+=1 ) {
  var x = document.getElementById("chunk_select");
  var option = document.createElement("option");
  option.text = conf.urls[i];
  option.value = conf.urls[i];
  x.add(option);
}
const connect = () => {
  try {
    const chunk_select = document.getElementById("chunk_select");
    const url = chunk_select.options[chunk_select.selectedIndex].value;
    log(`Connecting to`)
    log(`  ${url}`)
    while (data.socket_pairs.length >= 1) {
      data.socket_pairs[0].reader.close()
      data.socket_pairs[0].writer.close()
      data.socket_pairs.shift()
    }
    data.socket_pairs.push({
      'reader': new WebSocket(url),
      'writer': new WebSocket(url)
    })
    setup_socket_pair(data.socket_pairs[data.socket_pairs.length - 1])
  } catch(error) {
    console.error(error)
  }
}
document.getElementById("chunk_select").addEventListener('change', (event) => {
  connect()
});
let logged_count = 0;
const log_x_time = (x, message) => {
  if (logged_count < x) {
    log(message);
  }
  logged_count += 1;
}
const log = (message) => {
  console.log(message);
  const textarea_logs = document.getElementById('logs')
  textarea_logs.value += message + '\n';
  textarea_logs.scrollTop = textarea_logs.scrollHeight;
}
const setup_socket_pair = (socket_pair) => {
  socket_pair.reader.addEventListener('open', function (event) {
      socket_pair.reader.send('Hello Server!')
      log(`[reader] connected: ${socket_pair.reader.url}`)
      start_render_loop()
      window.onbeforeunload = function() {
          socket_pair.reader.onclose = function () {};
          socket_pair.reader.close();
      };
  });
  socket_pair.reader.addEventListener('close', function (event) {
      log(`[reader] connection closed: ${socket_pair.reader.url}`)
  });
  socket_pair.reader.addEventListener('error', function (event) {
      console.log('[reader] error')
  });
  socket_pair.reader.addEventListener('message', (event) => {
    chunk = JSON.parse(event.data)
  });

  socket_pair.writer.addEventListener('open', function (event) {
      socket_pair.writer.send('writer')
      log(`[writer] connected: ${socket_pair.writer.url}`)
      window.onbeforeunload = function() {
          socket_pair.writer.onclose = function () {};
          socket_pair.writer.close();
      };
  });
  socket_pair.writer.addEventListener('close', function (event) {
      log(`[writer] connection closed for`)
      log(`  ${socket_pair.reader.url}`)
  });
  socket_pair.writer.addEventListener('error', function (event) {
      console.log('[writer] error')
  });
  socket_pair.writer.addEventListener('message', (event) => {
    // chunk = JSON.parse(event.data)
  });
}
const start_render_loop = () => {
  log(`starting rendering`)
  render_loop()
}
const render_loop = () => {
  render()
  setTimeout(render_loop, 0)
}
const tohhmmssms = (duration_second) => {
    var sec_num = parseInt(duration_second, 10);
    var hours   = Math.floor(sec_num / 3600);
    var minutes = Math.floor((sec_num - (hours * 3600)) / 60);
    var seconds = sec_num - (hours * 3600) - (minutes * 60);
    var ms = parseFloat(duration_second, 10) - sec_num;
    if (hours   < 10) {hours   = "0"+hours;}
    if (minutes < 10) {minutes = "0"+minutes;}
    if (seconds < 10) {seconds = "0"+seconds;}
    return hours+':'+minutes+':'+seconds+'.'+ms;
}
const tohhmmss = (duration_second) => {
    var sec_num = parseInt(duration_second, 10);
    var hours   = Math.floor(sec_num / 3600);
    var minutes = Math.floor((sec_num - (hours * 3600)) / 60);
    var seconds = sec_num - (hours * 3600) - (minutes * 60);
    if (hours   < 10) {hours   = "0"+hours;}
    if (minutes < 10) {minutes = "0"+minutes;}
    if (seconds < 10) {seconds = "0"+seconds;}
    return hours+':'+minutes+':'+seconds;
}
let last_render_time_ms = Date.now();
let now_ms = Date.now();
let fps_list = []
const render = () => {
  if (!chunk.constants) {
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
  document.getElementById('fps').innerHTML = fps.toFixed(2);
  //
  const zoom = parseFloat(document.querySelector("#slider_1").value) / 1000.0 * 9.0 + 1.0
  const simulation_time_s = (chunk.step * chunk.constants.delta_time)
  const real_time_s = (chunk.real_time_ms / 1000)
  const simulation_speed = simulation_time_s / real_time_s
  document.getElementById('step').innerHTML = chunk.step;
  document.getElementById('simulation_time').innerHTML = tohhmmss(simulation_time_s);
  document.getElementById('real_time').innerHTML = tohhmmss(real_time_s);
  document.getElementById('simulation_speed').innerHTML = simulation_speed;
  document.getElementById('particles_count').innerHTML = chunk.particles_count;
  document.getElementById('entities_count').innerHTML = chunk.entities_count;
  document.getElementById('links_count').innerHTML = chunk.links_count;
  document.getElementById('total_energy').innerHTML = chunk.total_energy;
  document.getElementById('simulation_current_speed').innerHTML = chunk.stats[chunk.stats.length-1].simulation_speed;
  document.getElementById('distance_traveled_as_fitness_function').checked = chunk.constants.use_distance_traveled_as_fitness_function
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
      let x = particle.x + particle.data.MouthData.direction.x * particle.diameter * 0.35;
      let y = particle.y + particle.data.MouthData.direction.y * particle.diameter * 0.35;
      if (Math.abs(particle.data.MouthData.direction.x) < 0.1 && Math.abs(particle.data.MouthData.direction.y)  < 0.1)
      {
        x = particle.x + 0.0 * particle.diameter * 0.35;
         y = particle.y - 1.0 * particle.diameter * 0.35;
      }
      draw_mouth(canvas_1, x, y, particle.diameter, zoom, center_x, center_y)
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
      const x = particle.x + particle.data.EyeData.direction.x * particle.diameter * 0.3;
      const y = particle.y + particle.data.EyeData.direction.y * particle.diameter * 0.3;
      draw_eye(canvas_1, x, y, particle.diameter, zoom, center_x, center_y)
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
  const left = canvas_2.width * (center_x  - 0.5 / zoom)
  const top = canvas_2.height * (center_y - 0.5  / zoom)
  const width = canvas_2.width / zoom
  const height = canvas_2.height / zoom
  context_2.strokeStyle = '#fff'
  context_2.beginPath();
  context_2.rect(left, top, width, height);
  context_2.stroke();
  render_stats_age();
  render_stats_distance();
}
const render_stats_distance = () => {
  let l = chunk.stats.length;
  let max_distance = chunk.best_dna_ever_by_distance_traveled.distance_traveled;
  let last_distance_alive = chunk.stats[l-1].best_dna_alive_by_age.distance_traveled;
  document.getElementById('best_ever_distance_traveled').innerHTML = max_distance;
  document.getElementById('best_alive_distance_traveled').innerHTML = last_distance_alive;
  const resolution = parseFloat(document.querySelector("#resolution").value)
  let step = l / resolution;
  for (let i = 0; i < l ; i += step) {
      let stat = chunk.stats[Math.trunc(i)]
      const x = stat.step / chunk.step * canvas_3.width;
      [
        'best_dna_alive_by_age',
        'best_dna_ever_by_age',
        'best_dna_alive_by_distance_traveled',
        'best_dna_ever_by_distance_traveled'
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
const draw_eye = (canvas, x, y, diameter, zoom, center_x, center_y) => {
  draw_disk(canvas, x, y, diameter * 0.6, zoom, center_x, center_y, conf.colors.eye.white)
  draw_disk(canvas, x, y, diameter * 0.45, zoom, center_x, center_y, conf.colors.eye.black)
}
const draw_mouth = (canvas, x, y, diameter, zoom, center_x, center_y) => {
  draw_disk(canvas, x, y, diameter * 0.6, zoom, center_x, center_y, conf.colors.mouth.black)
}
const draw_body = (canvas, x, y, diameter, zoom, center_x, center_y) => {
  draw_disk(canvas, x, y, diameter, zoom, center_x, center_y, conf.colors.body)
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
  const r = 255.0;
  const g = (1.0 - output) * 255.0;
  const b = (1.0 - output) * 255.0;
  // const a = 0.5;
  const color = `rgb(${r}, ${g}, ${b})`
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
let chunk = {}
//
const canvas_1 = document.querySelector("#canvas_1")
canvas_1.width = window.innerHeight
canvas_1.height = window.innerHeight
const context_1 = canvas_1.getContext('2d')
//
const canvas_2 = document.querySelector("#canvas_2")
canvas_2.width = 250;
canvas_2.height = canvas_2.width
const context_2 = canvas_2.getContext('2d')
let center_x = 0.5;
let center_y = 0.5
let mousedown = false
canvas_2.onmousedown = function(e){
  mousedown = true
  const p = get_canvas_cursor_position(canvas_2, e)
  center_x = p.x / canvas_2.width
  center_y = p.y / canvas_2.height
}
canvas_2.onmousemove = function(e){
  if (mousedown) {
    const p = get_canvas_cursor_position(canvas_2, e)
    center_x = p.x / canvas_2.width
    center_y = p.y / canvas_2.height
  }
  console.log(center_x, center_y)
}
document.body.onmouseup = function(e){
  mousedown = false
}
//
const canvas_3 = document.querySelector("#canvas_3")
canvas_3.width = 250;
canvas_3.height = canvas_3.width
const context_3 = canvas_3.getContext('2d')
//
const canvas_4 = document.querySelector("#canvas_4")
canvas_4.width = 250;
canvas_4.height = canvas_4.width
const context_4 = canvas_4.getContext('2d')
//
document.getElementById('show_health').checked = true
//
document.querySelector('#use_distance_traveled_as_fitness_function').addEventListener('click', (event) => {
  data.socket_pairs[0].writer.send('use_distance_traveled_as_fitness_function')
});
document.querySelector('#use_distance_traveled_as_fitness_function_false').addEventListener('click', (event) => {
  data.socket_pairs[0].writer.send('use_distance_traveled_as_fitness_function_false')
});
//
connect()
