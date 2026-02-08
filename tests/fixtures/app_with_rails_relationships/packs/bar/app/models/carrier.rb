class Carrier < ActiveRecord::Base
  has_many :censuses
  has_many :tacos
  # Test that class_name: Foo.name is handled correctly
  # This should resolve to Census, NOT MyWidget (because class_name overrides)
  belongs_to :my_widget, class_name: Census.name, optional: true
end
