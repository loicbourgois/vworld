console.log('VWorld generation')
console.log(`  configuration_folder:  ${process.env.configuration_folder}`)
const fs = require('fs')
const configuration_path = `${process.env.configuration_folder}/configuration.json`
const configuration_str = fs.readFileSync(configuration_path, 'utf8')
console.log(`  configuration_path:    ${configuration_path}`)
const configuration = JSON.parse(configuration_str)
let port = 10000
let docker_run_string = ''
let k = 0
for (let i = configuration.start.x ; i < configuration.start.x + configuration.chunk_width ; i++) {
  for (let j = configuration.start.y ; j < configuration.start.y + configuration.chunk_height ; j++) {
    server_configuration = configuration.servers[`${i},${j}`]
    server_configuration.x = i
    server_configuration.y = j
    server_configuration.constants = configuration.constants
    server_configuration.thread_count = configuration.thread_count
    server_configuration_str = JSON.stringify(server_configuration)
    detach = ''
    if (configuration.width * configuration.height > 1) {
      detach = '--detach'
    }
    k += 1
    const name = `vworld-server-${k}`
    port += 1
    docker_run_string += `# ${name}
      docker rm -f ${name};
      docker run \\
        ${detach} \\
        --tty \\
        --env vworld_address=0.0.0.0 \\
        --env vworld_port=${port} \\
        --env vworld_chunk_configuration='${server_configuration_str}' \\
        --publish ${port}:${port} \\
        --name "${name}" \\
        "vworld-server";
    `
  }
}
file_path = `${process.env.configuration_folder}/docker-run.sh`
fs.writeFileSync(file_path, docker_run_string)
console.log(`updated ${file_path}`)
