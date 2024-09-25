module Foo
  class Creator
    sig { returns(String) }
    def self.build_foo
      "foo"
    end
  end
end
