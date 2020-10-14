console.log('VWorld generation')
console.log(`  configuration_folder:  ${process.env.configuration_folder}`)
const fs = require('fs')
const configuration_path = `${process.env.configuration_folder}/configuration.json`
const configuration_str = fs.readFileSync(configuration_path, 'utf8')
console.log(`  configuration_path:    ${configuration_path}`)
const configuration = JSON.parse(configuration_str)
chunk_configuration = configuration.servers[`${process.env.x},${process.env.y}`]
chunk_configuration.x = parseInt(process.env.x)
chunk_configuration.y = parseInt(process.env.y)
chunk_configuration.constants = configuration.constants
chunk_configuration_str = JSON.stringify(chunk_configuration, null, 2)
file_path = `${process.env.configuration_folder}/${process.env.x}:${process.env.y}.chunk.json`
fs.writeFileSync(file_path, chunk_configuration_str)
console.log(`updated ${file_path}`)
