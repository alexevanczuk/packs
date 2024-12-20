module Foo
  class Api
    def do_thing
      [
        Bar::Api,
        Bar::Api2,
        Bar::Api3,
      ]
    end
  end
end
