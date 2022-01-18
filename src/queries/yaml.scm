(block_mapping_pair
    key: (
      flow_node (
        (double_quote_scalar)? @translation_key
        (single_quote_scalar)? @translation_key
        (plain_scalar (string_scalar)?)? @translation_key
      )
    )
    value: [
      (
        flow_node (
          (double_quote_scalar)? @translation_value
          (single_quote_scalar)? @translation_value
          (plain_scalar (string_scalar)?)? @translation_value
        )
      )
      (
        block_node (
          (block_scalar) @translation_value
        )
      )
      (
        block_node (
          (block_sequence
            (block_sequence_item
              (flow_node
                (plain_scalar (string_scalar) @translation_value)
              )
            )
          )
        )
      )

    ]
  )* @translation_group

(boolean_scalar) @translation_error
