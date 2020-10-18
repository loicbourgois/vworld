const parse_chunk_data = (data) => {
  console.log('bdata', binary_data)
  const buffer = new ArrayBuffer(binary_data.byteLength);
  const view = new Uint8Array(buffer);
  console.log('v', view)
  return {
    step: view[0],
    constants: {
      'delta_time': view[1],
    },
  }
}
