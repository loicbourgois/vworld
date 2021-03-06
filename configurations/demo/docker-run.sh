# vworld-server-1
      docker rm -f vworld-server-1;
      docker run \
        --detach \
        --tty \
        --env vworld_address=0.0.0.0 \
        --env vworld_port=10001 \
        --env vworld_chunk_configuration='{"entities":[],"particles":[{"type_":"sun","x":0.5,"y":0.5}],"x":0,"y":0,"constants":{"use_distance_traveled_as_fitness_function":false,"display_simulation_logs":false,"muscles_use_output":false,"collision_push_rate":0.2,"drag_coefficient":10,"link_length_coefficient":1.01,"link_strengh_default":10.5,"diameter_muscle_change_rate":0.1,"delta_time":0.01,"default_mass":0.1,"default_diameter":0.01,"min_body_parts_count":10,"max_body_parts_count":10,"max_stats_count":1000,"plant":{"mutation_rate":1,"max_mutation_strength":0.05,"new_dna_rate":0,"min_count":1000,"energy_drop_rate_per_tick":0,"energy_drop_rate_per_tick_circle":0},"bloop":{"gene_random_mutation_rate":0.02,"gene_progressive_mutation_rate":1,"gene_progressive_mutation_strength":0.05,"new_dna_rate":0.1,"min_count":100,"energy_drop_rate_per_tick":0.0001,"starting_energy":0.2,"max_contraction":0.5},"gravity":{"x":0,"y":0},"energy_max":1,"energy_min":0,"eye_sight_length":0.02,"mouth_energy_eating_rate_per_second":1,"enable_auto_link_6":true,"destroy_unstable_entities":true},"thread_count":"auto"}' \
        --publish 10001:10001 \
        --name "vworld-server-1" \
        "vworld-server";
    