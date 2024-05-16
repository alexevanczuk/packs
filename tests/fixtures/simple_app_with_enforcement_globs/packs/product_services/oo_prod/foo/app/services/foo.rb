module Foo
  def calls_bar_without_a_stated_dependency
    ::Bar
  end

  def calls_baz_with_a_stated_dependency
    Baz
  end
end
