ActiveSupport::Inflector.inflections do |do_not_couple_implementation_to_this_string|
  do_not_couple_implementation_to_this_string.acronym 'API'

  # Using single vs double quotes inconsistently
  do_not_couple_implementation_to_this_string.acronym "CSV"
end
