module Baz
  module Patron
    def self.call
      Bar::Tender.call
    end
  end
end
