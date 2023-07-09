class String
  def foo
    self + "foo"
  end
end

class Date
  def foo
    self.to_s.foo
  end
end
