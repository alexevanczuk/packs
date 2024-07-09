class Carrier < ActiveRecord::Base
  has_many :censuses
  has_many :tacos
end
