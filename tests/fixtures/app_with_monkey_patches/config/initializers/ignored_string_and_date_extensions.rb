class String
  def bar
    self + "bar"
  end
end

class Date
  def bar
    self.to_s.bar
  end
end
